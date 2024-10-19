use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// INTERCEPTIONS.
/// CORS interception configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InterceptionCors {
    pub enable: bool,
    /// Allow origins
    pub allow_origins: Option<Vec<String>>,
    /// Allow headers
    pub allow_headers: Option<Vec<String>>,
    /// Allow methods
    pub allow_methods: Option<Vec<String>>,
    /// Max age
    pub max_age: Option<u64>,
}

/// A generic interception configuration that can be enabled or
/// disabled.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InterceptionCompression {
    pub enable: bool,
}

/// Timeout interception configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InterceptionTimeoutRequest {
    pub enable: bool,
    // Timeout request in milliseconds
    pub timeout: u64,
}

/// Limit payload size interception configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InterceptionLimitPayload {
    pub enable: bool,
    /// Body limit. for example: 5mb
    pub body_limit: String,
}

/// Static asset interception configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InterceptionStaticAssets {
    pub enable: bool,
    /// Check that assets must exist on disk
    pub must_exist: bool,
    /// Assets location
    pub folder: InterceptionFolderAssets,
    /// Fallback page for a case when no asset exists (404). Useful for SPA
    /// (single page app) where routes are virtual.
    pub fallback: String,
    /// Enable `precompressed_gzip`
    #[serde(default = "bool::default")]
    pub precompressed: bool,
}

/// Asset folder config.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InterceptionFolderAssets {
    /// Uri for the assets
    pub uri: String,
    /// Path for the assets
    pub path: String,
}

/// Server middleware configuration structure.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Interceptions {
    /// Setting cors configuration
    pub cors: Option<InterceptionCors>,
    /// Middleware that enable compression for the response.
    pub compression: Option<InterceptionCompression>,
    /// Middleware that limit the payload request.
    pub limit_payload: Option<InterceptionLimitPayload>,
    /// Setting a global timeout for the requests
    pub timeout_request: Option<InterceptionTimeoutRequest>,
    /// Serving static assets
    #[serde(rename = "static")]
    pub static_assets: Option<InterceptionStaticAssets>,
}

// APPLICATION CONFIGURATIONS.

/// Adapters configuration
///
/// Example (development): To configure settings for oauth2 or custom view
/// engine
/// ```yaml
/// # config/development.yaml
/// adapters:
///  oauth2:
///   authorization_code: # Authorization code grant type
///     - client_identifier: google # Identifier for the `OAuth2` provider.
///       Replace 'google' with your provider's name if different, must be
///       unique within the oauth2 config. ... # other fields
pub type Adapters = BTreeMap<String, serde_json::Value>;

/// Application's specific settings to expose `port`,
/// `host`, `protocol`, and possible url of the application
/// during and after development
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Server {
    pub port: u16,
    pub host: String,
    pub base_url: String,
    pub protocol: String,
    pub interceptions: Interceptions,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Secret {
    // APP_SECRET__COOKIE
    pub cookie: String,
    pub token_expiration: i64,
    pub cookie_expiration: i64,
}

/// The possible runtime environment for our application.
#[derive(Deserialize, Clone, PartialEq, Eq)]
pub enum Environment {
    #[serde(rename = "development")]
    Development,
    #[serde(rename = "production")]
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Development => "development",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "development" => Ok(Self::Development),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `development` or `production`.",
                other
            )),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Logger {
    /// Enable log write to stdout
    pub enable: bool,
    /// Set the logger level.
    ///
    /// * options: `trace` | `debug` | `info` | `warn` | `error`
    pub level: String,
}

/// Global settings for the exposing all preconfigured variables
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: Server,
    pub secret: Secret,
    pub logger: Logger,
    /// Custom app settings
    ///
    /// Example:
    /// ```yaml
    /// settings:
    ///   jwt:
    ///     secret: xxxxx
    ///     expiration: 10
    /// ```
    /// And then optionally deserialize it to your own `Configurations` type by
    /// accessing `ctx.settings.configurations`.
    #[serde(default)]
    pub settings: Option<serde_json::Value>,
    pub adapters: Option<Adapters>,
}

/// Multipurpose function that helps detect the current environment the application
/// is running using the `APP_ENVIRONMENT` environment variable.
///
/// APP_ENVIRONMENT = development | production.
///
/// After detection, it loads appropriate .yaml file
/// then it loads environment variable that override whatever is set in the .yaml file.
/// For this to work, you the environment variable MUST be in uppercase and starts with `APP`,
/// a `_` separator then the category of settings,
/// followed by `__` separator,  and then the variable, e.g.
/// `APP_APPLICATION__PORT=5001` for `port` to be set as `5001`
pub fn load_configuration(environment: &Environment) -> Result<Config, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let config_directories = base_path.join("configs");

    let environment_filename = format!("{}.yaml", environment.as_str());
    let cfg = config::Config::builder()
        .add_source(config::File::from(config_directories.join("base.yaml")))
        .add_source(config::File::from(
            config_directories.join(environment_filename),
        ))
        // Add in settings from environment variables (with a prefix of APP and '__' as separator)
        // E.g. `APP_APPLICATION__PORT=5001 would set `Settings.application.port`
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;
    cfg.try_deserialize::<Config>()
}
