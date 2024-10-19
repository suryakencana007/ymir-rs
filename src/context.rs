use crate::config::{Config, Environment};

#[derive(Clone)]
pub struct Context {
    /// The environment in which the application is running.
    pub environment: Environment,
    /// Settings for the application.
    pub configs: Config,
}
