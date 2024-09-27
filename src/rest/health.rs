use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use serde::Serialize;
use sidekiq::redis_rs::cmd;

use crate::{
    hooks::Context,
    job::{Pool, RedisConnectionManager},
    Result,
};

use super::responses::Json;

#[derive(Serialize)]
struct Health {
    pub ok: bool,
}

async fn ping() -> Result<Response> {
    Ok(Json(Health { ok: true }).into_response())
}

async fn health(State(ctx): State<Context>) -> Result<Response> {
    let mut is_health = match ctx.db.unwrap().ping().await {
        Ok(()) => true,
        Err(error) => {
            tracing::error!(err.msg = %error, err.detail = ?error, "health_db_ping_error");
            false
        }
    };
    if let Some(pool) = ctx.queue {
        if let Err(error) = redis_ping(&pool).await {
            tracing::error!(err.msg = %error, err.detail = ?error, "health_redis_ping_error");
            is_health = false
        }
    }
    Ok(Json(Health { ok: is_health }).into_response())
}

/// Run Redis ping command
async fn redis_ping(pool: &Pool<RedisConnectionManager>) -> Result<()> {
    let mut conn = pool.get().await?;
    Ok(cmd("PING")
        .query_async::<_, ()>(conn.unnamespaced_borrow_mut())
        .await?)
}

pub fn register_handler(app: Router<Context>) -> Router<Context> {
    app.route("/_ping", get(ping))
        .route("/_health", get(health))
}
