pub mod adapter;
pub mod boot;
mod errors;
pub mod hooks;
pub mod job;
pub mod logo;
pub mod prelude;
pub mod rest;
pub mod settings;
pub mod state;

pub type Result<T, E = errors::Error> = std::result::Result<T, E>;
pub type Error = errors::Error;
