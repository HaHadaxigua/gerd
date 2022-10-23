#[cfg(test)]
mod tests {
	use std::cell;
	use super::*;

	#[test]
	fn distinguish() {
		use rand;
		// &'static T is an immutable reference to some T, including up until the end of the program.
		// This is only possible if T itself is immutable and does not move after the reference was created.
		// We make it at the cost of memory laking.
		fn rand_str_generator() -> &'static str {
			let rand_string = rand::random::<u64>().to_string();
			Box::leak(rand_string.into_boxed_str())
		}

		// T: 'static is some T that can be safely held indefinitely long,
		// including up until the end of the program.
		// T: 'static should be read as "T is bounded by a 'static lifetime"
		fn drop_static<T: 'static>(t: T) {
			std::mem::drop(t);
			struct Ref<'a, T: 'a>(&'a T);
		}
	}

	#[test]
	fn test_lifetime() {
		#[derive(Debug)]
		struct NumRef<'a>(&'a i32);

		impl<'a> NumRef<'a> {
			// my struct is generic over 'a so that means I need to annotate
			// my self parameters with 'a too, right? (answer: no, not right)
			fn some_method(&mut self) {}
		}

		let mut num_ref = NumRef(&5);
		num_ref.some_method(); // mutably borrows num_ref for the rest of its lifetime
		num_ref.some_method(); // ❌
		println!("{:?}", num_ref); // ❌
	}

	#[test]
	fn closures() {
		fn call_with_ref<F>(some_closure: F) -> i32
			where F: for<'a> Fn(&'a i32) -> i32 {
			let value = 0;
			some_closure(&value)
		}

		fn call_with_one(some_closure: &dyn Fn(i32) -> i32) -> i32 {
			some_closure(1)
		}

		let answer = call_with_one(&|x| x + 2);
	}
}
