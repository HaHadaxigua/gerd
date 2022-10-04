use std::sync::{Mutex, Arc};
use std::thread;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_mutex() {
		let counter = Arc::new(Mutex::new(5));
		let mut handles = vec![];

		for _ in 0..10 {
			let counter = Arc::clone(&counter);
			let handle = thread::spawn(move || {
				let mut num = counter.lock().unwrap();

				*num += 1;
			});
			handles.push(handle);
		}

		for handle in handles {
			handle.join().unwrap();
		}

		println!("Result = {:?}", *counter.lock().unwrap());
	}
}