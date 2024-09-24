pub mod adapter;
pub mod boot;
pub mod errors;
pub mod hooks;
pub mod logo;
pub mod rest;
pub mod settings;

pub type Result<T, E = errors::Error> = std::result::Result<T, E>;
