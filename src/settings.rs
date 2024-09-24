use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Database {
    /// The URI for connecting to the database. For example:
    /// * Postgres: `postgres://root:12341234@localhost:5432/myapp_development`
    /// * Sqlite: `sqlite://db.sqlite?mode=rwc`
    pub uri: String,

    /// Enable `SQLx` statement logging
    pub enable_logging: bool,

    /// Minimum number of connections for a pool
    pub min_connections: u32,

    /// Maximum number of connections for a pool
    pub max_connections: u32,

    /// Set the timeout duration when acquiring a connection
    pub connect_timeout: u64,

    /// Set the idle duration before closing a connection
    pub idle_timeout: u64,

    /// Set the timeout for acquiring a connection
    pub acquire_timeout: Option<u64>,

    /// Run migration up when application loads. It is recommended to turn it on
    /// in development. In production keep it off, and explicitly migrate your
    /// database every time you need.
    #[serde(default)]
    pub auto_migrate: bool,

    /// Truncate database when application loads. It will delete data from your
    /// tables. Commonly used in `test`.
    #[serde(default)]
    pub dangerously_truncate: bool,

    /// Recreate schema when application loads. Use it when you want to reset
    /// your database *and* structure (drop), this also deletes all of the data.
    /// Useful when you're just sketching out your project and trying out
    /// various things in development.
    #[serde(default)]
    pub dangerously_recreate: bool,
}

/// Application's specific settings to expose `port`,
/// `host`, `protocol`, and possible url of the application
/// during and after development
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Server {
    pub port: u16,
    pub host: String,
    pub base_url: String,
    pub protocol: String,
    pub middlewares: Middlewares,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Secret {
    // APP_SECRET__COOKIE
    pub cookie: String,
    pub token_expiration: i64,
    pub cookie_expiration: i64,
}

/// Global settings for the exposing all preconfigured variables
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
    pub server: Server,
    pub database: Database,
    pub logger: Logger,
    pub frontend_url: String,
    pub secret: Secret,
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

/// Server middleware configuration structure.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Middlewares {
    /// Setting cors configuration
    pub cors: Option<CorsMiddleware>,
    /// Middleware that enable compression for the response.
    pub compression: Option<CompressionMiddleware>,
    /// Middleware that limit the payload request.
    pub limit_payload: Option<LimitPayloadMiddleware>,
    /// Setting a global timeout for the requests
    pub timeout_request: Option<TimeoutRequestMiddleware>,
    /// Serving static assets
    #[serde(rename = "static")]
    pub static_assets: Option<StaticAssetsMiddleware>,
}

/// CORS middleware configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CorsMiddleware {
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

/// A generic middleware configuration that can be enabled or
/// disabled.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompressionMiddleware {
    pub enable: bool,
}

/// Timeout middleware configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimeoutRequestMiddleware {
    pub enable: bool,
    // Timeout request in milliseconds
    pub timeout: u64,
}

/// Limit payload size middleware configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LimitPayloadMiddleware {
    pub enable: bool,
    /// Body limit. for example: 5mb
    pub body_limit: String,
}

/// Static asset middleware configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StaticAssetsMiddleware {
    pub enable: bool,
    /// Check that assets must exist on disk
    pub must_exist: bool,
    /// Assets location
    pub folder: FolderAssetsMiddleware,
    /// Fallback page for a case when no asset exists (404). Useful for SPA
    /// (single page app) where routes are virtual.
    pub fallback: String,
    /// Enable `precompressed_gzip`
    #[serde(default = "bool::default")]
    pub precompressed: bool,
}

/// Asset folder config.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FolderAssetsMiddleware {
    /// Uri for the assets
    pub uri: String,
    /// Path for the assets
    pub path: String,
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
pub fn get_settings(environment: &Environment) -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let settings_directory = base_path.join("settings");

    let environment_filename = format!("{}.yaml", environment.as_str());
    let settings = config::Config::builder()
        .add_source(config::File::from(settings_directory.join("base.yaml")))
        .add_source(config::File::from(
            settings_directory.join(environment_filename),
        ))
        // Add in settings from environment variables (with a prefix of APP and '__' as separator)
        // E.g. `APP_APPLICATION__PORT=5001 would set `Settings.application.port`
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    settings.try_deserialize::<Settings>()
}
