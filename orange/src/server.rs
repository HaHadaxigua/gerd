use crate::db::{DB, DbDropGuard};
use crate::connection::Connection;
use crate::shutdown::Shutdown;

use std::future::Future;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Semaphore};
use tracing::{debug, error, info, instrument};


#[derive(Debug)]
struct Listener {
	db_holder: DbDropGuard,
	listener: TcpListener,
	/// Limit the max number of connections.
	///
	/// A `Semaphore` is used to limit the max number of connections.
	limit_connections: Arc<Semaphore>,

	/// Broadcasts a shutdown signal to all active connections.
	notify_shutdown: broadcast::Sender<()>,

	/// Used as part of the graceful shutdown process to wait for client
	/// connections to complete processing.
	shutdown_complete_rx: mpsc::Receiver<()>,
	shutdown_complete_tx: mpsc::Sender<()>,
}

/// Per-connection handler. Reads requests from `connection` and applies the
/// commands to `db`.
struct Handler {
	db: DB,
	connection: Connection,
	shutdown: Shutdown,
	/// Not used directly. Instead, when `Handler` is dropped...?
	_shutdown_complete: mpsc::Sender<()>,
}

/// Maximum number of concurrent connections the redis server will accept.
///
/// When this limit is reached, the server will stop accepting connections until
/// an active connection terminates.
///
/// A real application will want to make this value configurable, but for this
/// example, it is hard coded.
///
/// This is also set to a pretty low value to discourage using this in
/// production (you'd think that all the disclaimers would make it obvious that
/// this is not a serious project... but I thought that about mini-http as
/// well).
const MAX_CONNECTIONS: usize = 250;

pub async fn run(listener: TcpListener, shutdown: impl Future) {
	let (notify_shutdown, _) = broadcast::channel(1);
	let (shutdown_complete_tx, shutdown_complete_rx) = mpsc::channel(1);

	// Initialize the listener state
	let mut server = Listener {
		listener,
		db_holder: DbDropGuard::new(),
		limit_connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
		notify_shutdown,
		shutdown_complete_tx,
		shutdown_complete_rx,
	};

	tokio::select! {
        res = server.run() => {
            // If an error is received here, accepting connections from the TCP
            // listener failed multiple times and the server is giving up and
            // shutting down.
            //
            // Errors encountered when handling individual connections do not
            // bubble up to this point.
            if let Err(err) = res {
                error!(cause = %err, "failed to accept");
            }
        }
        _ = shutdown => {
            // The shutdown signal has been received.
            info!("shutting down");
        }
    }

	// Extract the `shutdown_complete` receiver and transmitter
	// explicitly drop `shutdown_transmitter`. This is important, as the
	// `.await` below would otherwise never complete.
	let Listener {
		mut shutdown_complete_rx,
		shutdown_complete_tx,
		notify_shutdown,
		..
	} = server;

	// When `notify_shutdown` is dropped, all tasks which have `subscribe`d will
	// receive the shutdown signal and can exit
	drop(notify_shutdown);
	// Drop final `Sender` so the `Receiver` below can complete
	drop(shutdown_complete_tx);

	// Wait for all active connections to finish processing. As the `Sender`
	// handle held by the listener has been dropped above, the only remaining
	// `Sender` instances are held by connection handler tasks. When those drop,
	// the `mpsc` channel will close and `recv()` will return `None`.
	let _ = shutdown_complete_rx.recv().await;
}

impl Listener {
	async fn run(&mut self) -> crate::Result<()> {
		info!("accepting inbound connections");

		loop {
			let permit = self
				.limit_connections
				.clone()
				.acquire_owned()
				.await
				.unwrap();

			// Accept a new socket. This will attempt to perform error handling.
			// The `accept` method internally attempts to recover errors, so an
			// error here is non-recoverable.
			let socket = self.accept().await?;

			// Create the necessary per-connection handler state.
			let mut handler = Handler {
				// Get a handle to the shared database.
				db: self.db_holder.db(),

				// Initialize the connection state. This allocates read/write
				// buffers to perform redis protocol frame parsing.
				connection: Connection::new(socket),

				// Receive shutdown notifications.
				shutdown: Shutdown::new(self.notify_shutdown.subscribe()),

				// Notifies the receiver half once all clones are
				// dropped.
				_shutdown_complete: self.shutdown_complete_tx.clone(),
			};

			// Spawn a new task to process the connections. Tokio tasks are like
			// asynchronous green threads and are executed concurrently.
			tokio::spawn(async move {
				// Process the connection. If an error is encountered, log it.
				if let Err(err) = handler.run().await {
					error!(cause = ?err, "connection error");
				}
				// Move the permit into the task and drop it after completion.
				// This returns the permit back to the semaphore.
				drop(permit);
			});
		}
	}

	/// Accept an inbound connection.
	async fn accept(&mut self) -> crate::Result<TcpStream> {}

}

impl Handler {
	/// Process a single connection.
	///
	/// Request frames are read from the socket and processed. Responses are
	/// written back to the socket.
	///
	/// Currently, pipelining is not implemented. Pipelining is the ability to
	/// process more than one request concurrently per connection without
	/// interleaving frames. See for more details:
	/// https://redis.io/topics/pipelining
	///
	/// When the shutdown signal is received, the connection is processed until
	/// it reaches a safe state, at which point it is terminated.
	#[instrument(skip(self))]
	async fn run(&mut self) -> crate::Result<()> {
		while !self.shutdown.is_shutdown() {
			let maybe_frame =tokio::select! {
				res = self.connection.read_frame() => res?,
                _ = self.shutdown.recv() => {
                    // If a shutdown signal is received, return from `run`.
                    // This will result in the task terminating.
                    return Ok(());
                }
			};

			// If `None` is returned from `read_frame()` then the peer closed
			// the socket. There is no further work to do and the task can be
			// terminated.
			let frame = match maybe_frame {
				Some(frame) => frame,
				None => return Ok(()),
			};

			// Convert the redis frame into a command struct. This returns an
			// error if the frame is not a valid redis command or it is an
			// unsupported command.
			let cmd = Command::from_frame(frame)?;

			// Logs the `cmd` object. The syntax here is a shorthand provided by
			// the `tracing` crate. It can be thought of as similar to:
			//
			// ```
			// debug!(cmd = format!("{:?}", cmd));
			// ```
			//
			// `tracing` provides structured logging, so information is "logged"
			// as key-value pairs.
			debug!(?cmd);

			// Perform the work needed to apply the command. This may mutate the
			// database state as a result.
			//
			// The connection is passed into the apply function which allows the
			// command to write response frames directly to the connection. In
			// the case of pub/sub, multiple frames may be send back to the
			// peer.
			cmd.apply(&self.db, &mut self.connection, &mut self.shutdown)
				.await?;
		}
		Ok(())
	}
}