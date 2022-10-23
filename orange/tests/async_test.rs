#[tokio::test]
async fn test_async() {
	let handle = tokio::spawn(async {
		"return value"
	});

	let out = handle.await.unwrap();
	println!("GOT {}", out);
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn match_guard() {
		let mut x = 4;
		let y = false;

		match x {
			ref mut x => {
				*x = 5;
				println!("{:?}???", x)
			}
			4 | 5 if y => println!("yes"),
			_ => println!("no"),
		}

		assert_eq!(x, 5)
	}
}