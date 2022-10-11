fn main() {
	println!("Hello, world!");
}

#[cfg(test)]
mod tests {
	use rand;

	fn drop_static<T: 'static>(t: T) {
		std::mem::drop(t);
	}

	// its immutable
	static NUM: i32 = 18;
	// its mutable
	static NUM2: i32 = 19;

	fn coerce_static<'a>(_: &'a i32) -> &'a i32 {
		&NUM
	}

	#[test]
	fn do_coerce() {
		{
			// Make an integer to use for `coerce_static`:
			let lifetime_num = 9;

			// Coerce `NUM` to lifetime of `lifetime_num`:
			let coerced_static = coerce_static(&lifetime_num);

			println!("coerced_static: {}", coerced_static);
		}

		println!("NUM: {} stays accessible!", NUM);
	}

	#[test]
	fn test_static() {
		let mut strings: Vec<String> = Vec::new();
		for _ in 0..10 {
			if rand::random() {
				// all the strings are randomly generated
				// and dynamically allocated at run-time
				let string = rand::random::<u64>().to_string();
				strings.push(string);
			}
		}

		// strings are owned types so they're bounded by 'static
		for mut string in strings {
			// all the strings are mutable
			string.push_str("a mutation");
			// all the strings are droppable
			drop_static(string); // ✅
		}

		// all the strings have been invalidated before the end of the program
		println!("I am the end of the program");
	}
}