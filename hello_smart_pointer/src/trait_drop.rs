use crate::MyBox;

/// Drop used for release resources automatically
impl<T> Drop for MyBox<T> {
	fn drop(&mut self) {
		println!("just do nothing")
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn try_early_free() {
		let custom_smart_pointer = MyBox::new("string");
		// custom_smart_pointer.drop()
		drop(custom_smart_pointer) // free resource earlier
	}
}
