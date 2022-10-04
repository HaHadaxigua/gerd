use std::sync::mpsc;
use std::thread;
use std::time::Duration;


#[cfg(test)]
mod tests {
	use super::*;

	// use channel
	#[test]
	fn test_mpsc() {
		let (tx, rx) = mpsc::channel();

		thread::spawn(move || {
			let val = String::from("hello");
			tx.send(val).unwrap(); // we lost the ownership here
			// println!("val is {}", val); // cannot use the sent value again

			let vals = vec![
				String::from("hi"),
				String::from("from"),
				String::from("the"),
				String::from("thread"),
			];

			for val in vals {
				tx.send(val).unwrap();
				thread::sleep(Duration::from_secs(1));
			}
		});

		let receiver = rx.recv().unwrap();
		println!("Get: {}", receiver);

		for received in rx {
			println!("Got: {}", received)
		}
	}
}