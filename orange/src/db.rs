use std::collections::{BTreeMap, HashMap};
use std::ops::Index;
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use tokio::sync::{broadcast, Notify};
use tokio::time::{self, Duration, Instant};
use tracing::debug;

/// A wrapper around a `Db` instance. This exists to allow orderly cleanup
/// of the `Db` by signalling the background purge task to shut down when
/// this struct is dropped.
#[derive(Debug)]
pub(crate) struct DbDropGuard {
	/// The `Db` instance that will be shut down when this `DbHolder` struct
	/// is dropped.
	db: DB,
}

/// A `Db` instance is a handle to shared state. Cloning `DB` is shallow and only
/// incurs an atomic ref count increment.
///
/// When a `Db` value is created, a background task is spawned. This task is
/// used to expire values after the requested duration has elapsed. The task
/// runs until all instances of `Db` are dropped, at which point the task
/// terminates.
#[derive(Debug, Clone)]
pub(crate) struct DB {
	/// Handle to shared state.
	/// The background task will also have an `Arc<Shared>`
	shared: Arc<Shared>,
}

#[derive(Debug)]
struct Shared {
	/// The shared state is guarded by a mutex.
	/// Handle to shared state.
	state: Mutex<State>,

	/// Notifies the background task handling entry expiration. The background
	/// task waits on this to be notified, then checks for expired values or the
	/// shutdown signal.
	background_task: Notify,
}

#[derive(Debug)]
struct State {
	/// The key-value data.
	entries: HashMap<String, Entry>,

	/// The pub/sub key-space. Redis uses a **separate** key space for key-value
	/// and pub/sub. `mini-redis` handles this by using a separate `HashMap`.
	pub_sub: HashMap<String, broadcast::Sender<Bytes>>,

	/// Tracks key TTLs.
	///
	/// A `BTreeMap` is used to maintain expirations sorted by when they expire.
	/// This allows the background task to iterate this map to find the value
	/// expiring next.
	///
	/// While highly unlikely, it is possible for more than one expiration to be
	/// created for the same instant. Because of this, the `Instant` is
	/// insufficient for the key. A unique expiration identifier (`u64`) is used
	/// to break these ties.
	expirations: BTreeMap<(Instant, u64), String>,

	/// Identifier to use for the next expiration. Each expiration is associated
	/// with a unique identifier. See above for why.
	next_id: u64,

	/// True when the Db instance is shutting down. This happens when all `Db`
	/// values drop. Setting this to `true` signals to the background task to
	/// exit.
	shutdown: bool,
}

/// Entry in the key-value store
#[derive(Debug)]
struct Entry {
	/// Uniquely identifies this entry.
	id: u64,

	/// Stored data
	data: Bytes,

	/// Instant at which the entry expires and should be removed from the
	/// database.
	expires_at: Option<Instant>,
}

impl DbDropGuard {
	/// Create a new `DbHolder`, wrapping a `Db` instance. When this is dropped
	/// the `Db`'s purge task will be shut down.
	pub(crate) fn new() -> DbDropGuard {
		DbDropGuard { db: DB::new() }
	}

	/// Get the shared database. Internally, this is an
	/// `Arc`, so a clone only increments the ref count.
	pub(crate) fn db(&self) -> DB {
		self.db.clone()
	}
}

impl Drop for DbDropGuard {
	fn drop(&mut self) {
		// Signal the 'Db' instance to shut down the task that purges expired keys
		self.db.shutdown_purge_task();
	}
}

impl DB {
	/// Create a new, empty, `Db` instance. Allocates shared state and spawns a
	/// background task to manage key expiration.
	pub(crate) fn new() -> DB {
		let shared = Arc::new(Shared {
			state: Mutex::new(State {
				entries: HashMap::new(),
				pub_sub: HashMap::new(),
				expirations: BTreeMap::new(),
				next_id: 0,
				shutdown: false,
			}),
			background_task: Notify::new(),
		});

		DB { shared }
	}

	/// Get the value associated with a key.
	///
	/// Returns `None` if there is no value associated with the key. This may be
	/// due to never having assigned a value to the key or a previously assigned
	/// value expired.
	pub(crate) fn get(&self, key: &str) -> Option<Bytes> {
		let state = self.shared.state.lock().unwrap();
		state.entries.get(key).map(|entry| entry.data.clone())
	}

	/// Set the value associated with a key along with an optional expiration
	/// Duration.
	///
	/// If a value is already associated with the key, it is removed.
	pub(crate) fn set(&self, key: String, value: Bytes, expire: Option<Duration>) {
		let mut state = self.shared.state.lock().unwrap();

		let id = state.next_id;
		state.next_id += 1;

		let mut notify = false;
		let expires_at = expire.map(|duration| {
			let when = Instant::now() + duration;

			notify = state
				.next_expiration()
				.map(|expiration| expiration > when)
				.unwrap_or(true);

			state.expirations.insert((when, id), key.clone());
			when
		});

		let prev = state.entries.insert(
			key,
			Entry {
				id,
				data: value,
				expires_at,
			},
		);

		if let Some(prev) = prev {
			if let Some(when) = prev.expires_at {
				state.expirations.remove(&(when, prev.id));
			}
		}

		// Release the mutex before notifying the background task. This helps
		// reduce contention by avoiding the background task waking up only to
		// be unable to acquire the mutex due to this function still holding it.
		drop(state);

		if notify {
			self.shared.background_task.notify_one();
		}
	}

	/// Returns a receiver for the requested channel
	pub(crate) fn subscribe(&self, key: String) -> broadcast::Receiver<Bytes> {
		use std::collections::hash_map::Entry;

		let mut state = self.shared.state.lock().unwrap();

		match state.pub_sub.entry(key) {
			Entry::Occupied(e) => e.get().subscribe(),
			Entry::Vacant(e) => {
				let (tx, rx) = broadcast::channel(1024);
				e.insert(tx);
				rx
			}
		}
	}

	/// Publish a message to the channel.
	/// Return the number of subscribers listening on the channel.
	pub(crate) fn publish(&self, key: &str, value: Bytes) -> usize {
		let state = self.shared.state.lock().unwrap();

		state
			.pub_sub
			.get(key)
			.map(|tx| tx.send(value).unwrap_or(0))
			.unwrap_or(0)
	}

	fn shutdown_purge_task(&self) {
		let mut state = self.shared.state.lock().unwrap();
		state.shutdown = true;

		drop(state);
		self.shared.background_task.notify_one();
	}
}

impl Shared {
	fn purge_expired_keys(&self) -> Option<Instant> {
		let mut state = self.state.lock().unwrap();

		if state.shutdown {
			return None;
		}

		// make the borrow checker happy.
		// actually, the state we get before is 'MutexGuard'
		let state = &mut *state;

		// Find all keys scheduled to expire **before** now.
		let now = Instant::now();

		while let Some((&(when, id), key)) = state.expirations.iter().next() {
			if when > now {
				return Some(when);
			}

			state.entries.remove(key);
			state.expirations.remove(&(when, id));
		}

		None
	}

	fn is_shutdown(&self) -> bool {
		self.state.lock().unwrap().shutdown
	}
}

impl State {
	fn next_expiration(&self) -> Option<Instant> {
		self.expirations
			.keys()
			.next()
			.map(|expiration| expiration.0)
	}
}

async fn purge_expired_tasks(shared: Arc<Shared>) {
	while !shared.is_shutdown() {
		if let Some(when) = shared.purge_expired_keys() {
			tokio::select! {
                _ = time::sleep_until(when) => {

				}
                _ = shared.background_task.notified() => {

				}
            }
		} else {
			shared.background_task.notified().await;
		}
	}

	debug!("Purge background task shut down")
}

