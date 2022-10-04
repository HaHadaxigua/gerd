use std::thread;
use std::time::Duration;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn concurrent_with_thread() {
		let handle = thread::spawn(|| {
			for i in 1..10 {
				println!("hi number {} from the spawned thread!", i);
				thread::sleep(Duration::from_millis(1));
			}
		});

		for i in 1..5 {
			println!("hi number {} from the main thread!", i);
			thread::sleep(Duration::from_millis(1));
		}

		handle.join().unwrap(); // call the join method to make current thread wait its thread finish.
		// join will block
	}

	#[test]
	fn try_move_with_closure() {
		let v = vec![1, 2, 3];
		let handle = thread::spawn(move || {
			println!("{:?}", v)
		});

		handle.join().unwrap();
	}
}