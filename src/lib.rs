mod error;
pub mod proto;

pub use error::*;

pub type Result<T, E = Error> = std::result::Result<T, E>;
