extern crate core;

mod deref;
mod trait_drop;
mod rc;
mod weak;

/// This is a tuple struct
struct MyBox<T>(T);

