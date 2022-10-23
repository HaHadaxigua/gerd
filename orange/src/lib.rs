extern crate core;

pub mod db;
pub mod server;
pub mod frame;
pub mod connection;
pub mod shutdown;


/// Error returned by most functions.
///
/// When writing a real application, one might want to consider a specialized
/// error handling crate or defining an error type as an `enum` of causes.
/// However, for our example, using a boxed `std::error::Error` is sufficient.
pub type Error = Box<dyn std::error::Error + Send + Sync>;


/// A specialized `Result` type for mini-redis operations.
///
/// This is defined as a convenience.
pub type Result<T> = std::result::Result<T, Error>;