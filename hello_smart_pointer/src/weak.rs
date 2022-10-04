use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Debug)]
enum List {
	Cons(i32, RefCell<Rc<List>>),
	// RefCell means we can modify the value of i32
	Nil,
}

impl List {
	fn tail(&self) -> Option<&RefCell<Rc<List>>> {
		match self {
			List::Cons(_, item) => Some(item),
			List::Nil => None,
		}
	}
}

#[derive(Debug)]
struct Node {
	value: i32,
	// when we drop children, parent doesn't need to be dropped.
	// a node can refer to his parent, but doesn't need to own his parent.
	// so we use weak reference here, to replace Rc
	parent: RefCell<Weak<Node>>,

	// we want to change some value of children, so we use Refcell
	children: RefCell<Vec<Rc<Node>>>, // we want to access the children Node, so use Rc
}

#[cfg(test)]
mod tests {
	use super::*;


	#[test]
	fn test_reference_circle() {
		let a = Rc::new(List::Cons(5, RefCell::new(Rc::new(List::Nil))));
		println!("a initial rc count = {}", Rc::strong_count(&a));
		println!("a next item = {:?}", a.tail());

		let b = Rc::new(List::Cons(10, RefCell::new(Rc::clone(&a))));
		println!("a rc count after b creation = {}", Rc::strong_count(&a));
		println!("b initial rc count = {}", Rc::strong_count(&b));
		println!("b next item = {:?}", b.tail());

		if let Some(link) = a.tail() {
			*link.borrow_mut() = Rc::clone(&b);
		}
		println!("b rc count after changing a = {}", Rc::strong_count(&b));
		println!("a rc count after changing a = {}", Rc::strong_count(&a));

		// println!("a next item = {:?}", a.tail());
	}

	#[test]
	fn test_weak() {
		let leaf = Rc::new(Node {
			value: 3,
			parent: RefCell::new(Weak::new()),
			children: RefCell::new(vec![]),
		});

		println!("leaf parent = {:?}", leaf.parent.borrow().upgrade());

		let branch = Rc::new(Node {
			value: 10,
			// the Node in leaf now has two owners: leaf and branch.
			parent: RefCell::new(Weak::new()),
			children: RefCell::new(vec![Rc::clone(&leaf)]),
		});

		*leaf.parent.borrow_mut() = Rc::downgrade(&branch);
		println!("leaf parent = {:?}", leaf.parent.borrow().upgrade());
	}
}