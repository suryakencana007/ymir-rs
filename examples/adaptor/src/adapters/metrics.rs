use async_trait::async_trait;
use axum::{
    http::Request,
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};
use ymir::{
    adapter::{Adapter, AdapterPriority, AdapterState},
    context::Context,
    Result,
};

/// Metrics Adapter Example
#[derive(Debug)]
pub struct MetricsAdapter {
    state: AdapterState,
    metrics_endpoint: String,
}

impl MetricsAdapter {
    pub fn new(metrics_endpoint: String) -> Self {
        Self {
            state: AdapterState::Initialized,
            metrics_endpoint,
        }
    }
}

#[async_trait]
impl Adapter for MetricsAdapter {
    fn name(&self) -> String {
        "MetricsAdapter".to_string()
    }

    fn priority(&self) -> AdapterPriority {
        AdapterPriority::Normal
    }

    fn state(&self) -> AdapterState {
        self.state.clone()
    }

    async fn after_route(&self, _ctx: &Context, router: Router) -> Result<Router> {
        // Add metrics middleware to all routes
        let metrics_middleware = |req: Request<axum::body::Body>, next: Next| async {
            let start = std::time::Instant::now();
            let response: Response = next.run(req).await;
            let duration = start.elapsed();

            // Record metrics here
            tracing::info!("Request processed in {:?}", duration);

            response
        };

        Ok(router
            .layer(middleware::from_fn(metrics_middleware))
            .route(&self.metrics_endpoint, get(|| async { "Metrics endpoint" })))
    }
}

#[cfg(test)]
mod tests {
    use ymir::adapter::AdapterManager;

    use super::*;
    // use tokio::sync::Mutex as TokioMutex;

    // Helper function to create a test context
    fn create_test_context() -> Context {
        // Your context creation logic
        Context::default()
    }

    #[tokio::test]
    async fn test_adapter_manager_lifecycle() {
        let ctx = create_test_context();
        let mut manager = AdapterManager::new(ctx);

        // Add test adapters

        let metrics_adapter = MetricsAdapter::new("/metrics".to_string());

        manager.register(Box::new(metrics_adapter));

        // Test initialization
        assert!(manager.init_all().await.is_ok());

        // Test route configuration
        let router = Router::new();
        let _ = manager.configure_routes(router).await.unwrap();

        // Test shutdown
        assert!(manager.stop_all().await.is_ok());
    }
}
