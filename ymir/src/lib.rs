pub mod adapter;
pub mod config;
pub mod context;
pub mod errors;
pub mod health;
pub mod hook;
pub mod interception;
pub(crate) mod logo;
pub mod prelude;
pub mod render;
pub mod responses;
pub mod signal;
pub mod startup;
pub mod state;
pub mod types;

pub type Result<T, E = errors::Error> = std::result::Result<T, E>;
