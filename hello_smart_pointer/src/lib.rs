use std::ops::{Deref, DerefMut};

/// This is a tuple struct
struct MyBox<T>(T);

impl<T> MyBox<T> {
	#[allow(dead_code)]
	fn new(x: T) -> MyBox<T> {
		MyBox(x)
	}
}

/// Deref trait for immutable variable
impl<T> Deref for MyBox<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

/// DerefMut trait for mutable variable
impl<T> DerefMut for MyBox<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		todo!()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn my_box_try() {
		let x = 5;
		let y = MyBox::new(x);

		assert_eq!(5, x);
		assert_eq!(5, *y);

		fn hello(name: &str) {
			assert_eq!(name, name);
		}

		hello(&MyBox::new(String::from("hello")))
	}
}