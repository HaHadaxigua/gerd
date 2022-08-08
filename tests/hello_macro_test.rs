use hello_macro::HelloMacro;
use hello_macro_derive::HelloMacro;

#[derive(HelloMacro)]
struct Pancakes;

#[derive(HelloMacro)]
struct Bread;

#[test]
fn test_impl_hello_macro() {
    Pancakes::hello_macro();
    Bread::hello_macro();
}