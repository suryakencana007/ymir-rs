<div align="center">
<h1>Ymir</h1>

<h3>Core library for building scalable application using rust.</h3>

[![crate](https://img.shields.io/crates/v/ymir.svg)](https://crates.io/crates/ymir)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)

</div>

# Ymir

## Ymir is a library to support your boilerplate application and help you to scalable it.

[![GitHub stars](https://img.shields.io/github/stars/suryakencana007/ymir-rs.svg?style=social&label=Star&maxAge=1)](https://github.com/suryakencana007/ymir-rs/stargazers/)
If you like what we do, consider starring, sharing and contributing!

## Getting Started

```cli
cargo add ymir
cargo add ymir-openapi
```

### Structure folder

```bash
myapp
+-- src
    +-- adapters
        ├── mod.rs
    +-- bin
        ├── main.rs
    +-- features
        ├── mod.rs
    ├── app.rs
    ├── lib.rs
├── .env (optional development)
├── .dockerignore
├── .gitignore
├── Cargo.toml
├── justfile
├── README.md
```

## Features

# Server Lifecycle Management System

## Core Components

### `LifeCycle` Trait
A trait defining the core lifecycle behavior for web applications:

```rust
#[async_trait]
pub trait LifeCycle {
    fn version() -> String;  // Optional, defaults to "dev"
    fn app_name() -> &'static str;  // Required, typically uses CARGO_CRATE_NAME
    async fn rest(ctx: &Context, app: Router) -> Result<()>;  // Server startup
    async fn adapters() -> Result<Vec<Box<dyn Adapter>>>;  // External adapters
    fn routes(ctx: Context) -> Router;  // Application routes
}
```

### Key Functions

#### `ymir::startup::run<L: LifeCycle>`
Starts the server for a given LifeCycle implementation:
```rust
pub async fn run<LC: LifeCycle>() -> Result<()> {
```

## Features

### Router init
Constructs the application router with:
- Base routes from LifeCycle implementation
- HTTP request tracing
- Static asset serving (optional)
- Custom middleware via interception

### Static Asset Serving
- Configurable through server settings
- Supports:
  - Custom base path and fallback file
  - Optional existence validation
  - Precompressed (gzip) assets
  - Custom URI mounting point

### Graceful Shutdown
- Handles both Ctrl+C and SIGTERM (Unix only)
- Implements graceful shutdown for clean server termination

## Configuration Example

```rust
// Implementation example
struct MyApp;

#[async_trait]
impl LifeCycle for MyApp {
    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    fn routes(ctx: Context) -> Router {
        Router::new()
            .route("/", get(|| async { "Hello World" }))
    }
}
```

Now `cd` into your `myapp` and start your app:
```sh
$ cargo run
#####################################

           ／＞　　フ
　　　 　　|  _　 _ l
　 　　 　／ヽ ミ＿xノ
　　 　 /　　　 　 |
　　　 /　 ヽ　　 ﾉ
　 　 │　　|　|　|
　／￣|　　 |　|　|
　| (￣ヽ＿_ヽ_)__)
　＼二つ

,-.,-.,-. .--.  .--. .-..-..-.
: ,. ,. :' '_.'' .; :: `; `; :
:_;:_;:_;`.__.'`.__.'`.__.__.'

#####################################

environment: development

listening on 127.0.0.1:5050
version: 0.3.0 (dev)
2024-11-07T16:51:39.926177Z  INFO ymir::adapter: init adapter adapters=""
2024-11-07T16:51:39.926317Z  INFO ymir::adapter: before run adapter adapters=""
2024-11-07T16:51:39.927364Z  INFO ymir::interception: [Middleware] +cors
2024-11-07T16:51:39.927495Z  INFO ymir::interception: [Middleware] +timeout
2024-11-07T16:51:39.927596Z  INFO ymir::interception: [Middleware] +compression
2024-11-07T16:51:39.927754Z  INFO ymir::interception: [Middleware] +limit payload data="7mb"
2024-11-07T16:51:39.927942Z  INFO ymir::adapter: after router adapter adapters=""
2024-11-07T16:51:39.929498Z  INFO ymir::hook: Listening on 127.0.0.1:5050
```

## License

Licensed under ([MIT license](LICENSE) or <http://opensource.org/licenses/MIT>)
