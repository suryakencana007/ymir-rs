use async_trait::async_trait;
use axum::Router;
use std::error::Error;
use std::fmt::Debug;
use tokio::sync::broadcast;

use crate::{context::Context, Result};

/// Represents the current state of an adapter
#[derive(Debug, Clone, PartialEq)]
pub enum AdapterState {
    Initialized,
    Running,
    Stopped,
    Failed,
}

/// Priority level for adapter execution order
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AdapterPriority {
    High = 0,
    Normal = 1,
    Low = 2,
}

impl Default for AdapterPriority {
    fn default() -> Self {
        Self::Normal
    }
}

// Enhanced Adapter trait
#[async_trait]
pub trait Adapter: Send + Sync + Debug {
    /// The Adapter name
    fn name(&self) -> String;

    /// Get the adapter's priority level
    fn priority(&self) -> AdapterPriority {
        AdapterPriority::default()
    }

    /// Get the current state of the adapter
    fn state(&self) -> AdapterState;

    /// Initialize the adapter
    async fn init(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called before the server starts running
    async fn before_run(&mut self, ctx: Context) -> Result<Context> {
        Ok(ctx.clone())
    }

    /// Called after routes are configured but before the server starts
    async fn after_route(&self, _ctx: &Context, router: Router) -> Result<Router> {
        Ok(router.clone())
    }

    /// Called when the server is shutting down
    async fn before_stop(&self, _ctx: &Context) -> Result<()> {
        Ok(())
    }

    /// Called after the server has stopped
    async fn after_stop(&self, _ctx: Context) -> Result<()> {
        Ok(())
    }

    /// Handle any errors that occur during adapter lifecycle
    async fn handle_error(&self, error: Box<dyn Error + Send + Sync>) -> Result<()> {
        Err(error.into())
    }
}

/// Manages the lifecycle of multiple adapters\
pub struct AdapterManager {
    adapters: Vec<Box<dyn Adapter>>,
    ctx: Context,
    shutdown_tx: broadcast::Sender<()>,
}

impl AdapterManager {
    pub fn new(ctx: Context) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            adapters: Vec::new(),
            ctx,
            shutdown_tx,
        }
    }

    /// Register a new adapter
    pub fn register(&mut self, adapter: Box<dyn Adapter>) {
        self.adapters.push(adapter);
        // Sort adapters by priority
        self.adapters.sort_by_key(|a| a.priority());
    }

    /// Initialize all registered adapters
    pub async fn init_all(&mut self) -> Result<()> {
        tracing::info!(adapters = ?self.adapters.iter().map(|init| init.name()).collect::<Vec<_>>().join(","), "init adapter");
        for adapter in &mut self.adapters {
            if let Err(e) = adapter.init().await {
                adapter.handle_error(Box::new(e)).await?;
            }
        }
        Ok(())
    }

    /// Run before_run on all adapters
    pub async fn before_run(&mut self) -> Result<Context> {
        tracing::info!(adapters = ?self.adapters.iter().map(|init| init.name()).collect::<Vec<_>>().join(","), "before run adapter");
        let mut current_ctx = self.ctx.clone();
        for adapter in &mut self.adapters {
            match adapter.before_run(current_ctx.clone()).await {
                Ok(ctx) => current_ctx = ctx,
                Err(e) => {
                    adapter.handle_error(Box::new(e)).await?;
                }
            }
        }
        Ok(current_ctx)
    }

    /// Configure routes through all adapters
    pub async fn configure_routes(&self, mut router: Router) -> Result<Router> {
        tracing::info!(adapters = ?self.adapters.iter().map(|init| init.name()).collect::<Vec<_>>().join(","), "after router adapter");
        for adapter in &self.adapters {
            match adapter.after_route(&self.ctx, router.clone()).await {
                Ok(r) => router = r,
                Err(e) => {
                    adapter.handle_error(Box::new(e)).await?;
                }
            }
        }
        Ok(router)
    }

    /// Gracefully stop all adapters
    pub async fn stop_all(&self) -> Result<()> {
        // Notify all adapters of impending shutdown
        let _ = self.shutdown_tx.send(());

        // Call before_stop on all adapters
        for adapter in &self.adapters {
            if let Err(e) = adapter.before_stop(&self.ctx).await {
                adapter.handle_error(Box::new(e)).await?;
            }
        }

        // Call after_stop on all adapters
        for adapter in &self.adapters {
            if let Err(e) = adapter.after_stop(self.ctx.clone()).await {
                adapter.handle_error(Box::new(e)).await?;
            }
        }

        Ok(())
    }

    /// Get a reference to the shutdown broadcast channel
    pub fn shutdown_signal(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[derive(Debug)]
    struct MockAdapter {
        name: String,
        state: AdapterState,
        priority: AdapterPriority,
        init_called: Arc<AtomicBool>,
        before_run_called: Arc<AtomicBool>,
        after_route_called: Arc<AtomicBool>,
    }

    impl MockAdapter {
        fn new(name: &str, priority: AdapterPriority) -> Self {
            Self {
                name: name.to_string(),
                state: AdapterState::Initialized,
                priority,
                init_called: Arc::new(AtomicBool::new(false)),
                before_run_called: Arc::new(AtomicBool::new(false)),
                after_route_called: Arc::new(AtomicBool::new(false)),
            }
        }
    }

    #[async_trait]
    impl Adapter for MockAdapter {
        fn name(&self) -> String {
            self.name.clone()
        }

        fn priority(&self) -> AdapterPriority {
            self.priority.clone()
        }

        fn state(&self) -> AdapterState {
            self.state.clone()
        }

        async fn init(&mut self) -> Result<()> {
            self.init_called.store(true, Ordering::SeqCst);
            Ok(())
        }

        async fn before_run(&mut self, ctx: Context) -> Result<Context> {
            self.before_run_called.store(true, Ordering::SeqCst);
            Ok(ctx)
        }

        async fn after_route(&self, _ctx: &Context, router: Router) -> Result<Router> {
            self.after_route_called.store(true, Ordering::SeqCst);
            Ok(router)
        }
    }

    #[tokio::test]
    async fn test_adapter_registration_and_priority_sorting() {
        let ctx = Context::default(); // Assuming Context has a default implementation
        let mut manager = AdapterManager::new(ctx);

        let adapter1 = MockAdapter::new("adapter1", AdapterPriority::Low);
        let adapter2 = MockAdapter::new("adapter2", AdapterPriority::High);
        let adapter3 = MockAdapter::new("adapter3", AdapterPriority::Normal);

        manager.register(Box::new(adapter1));
        manager.register(Box::new(adapter2));
        manager.register(Box::new(adapter3));

        // Check if adapters are sorted by priority
        assert_eq!(manager.adapters[0].priority(), AdapterPriority::High);
        assert_eq!(manager.adapters[1].priority(), AdapterPriority::Normal);
        assert_eq!(manager.adapters[2].priority(), AdapterPriority::Low);
    }

    #[tokio::test]
    async fn test_init_all() {
        let ctx = Context::default();
        let mut manager = AdapterManager::new(ctx);

        let adapter1 = MockAdapter::new("adapter1", AdapterPriority::Normal);
        let init_called1 = adapter1.init_called.clone();

        let adapter2 = MockAdapter::new("adapter2", AdapterPriority::Normal);
        let init_called2 = adapter2.init_called.clone();

        manager.register(Box::new(adapter1));
        manager.register(Box::new(adapter2));

        manager.init_all().await.unwrap();

        assert!(init_called1.load(Ordering::SeqCst));
        assert!(init_called2.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_before_run() {
        let ctx = Context::default();
        let mut manager = AdapterManager::new(ctx);

        let adapter1 = MockAdapter::new("adapter1", AdapterPriority::Normal);
        let before_run_called1 = adapter1.before_run_called.clone();

        let adapter2 = MockAdapter::new("adapter2", AdapterPriority::Normal);
        let before_run_called2 = adapter2.before_run_called.clone();

        manager.register(Box::new(adapter1));
        manager.register(Box::new(adapter2));

        manager.before_run().await.unwrap();

        assert!(before_run_called1.load(Ordering::SeqCst));
        assert!(before_run_called2.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_configure_routes() {
        let ctx = Context::default();
        let mut manager = AdapterManager::new(ctx);

        let adapter1 = MockAdapter::new("adapter1", AdapterPriority::Normal);
        let after_route_called1 = adapter1.after_route_called.clone();

        let adapter2 = MockAdapter::new("adapter2", AdapterPriority::Normal);
        let after_route_called2 = adapter2.after_route_called.clone();

        manager.register(Box::new(adapter1));
        manager.register(Box::new(adapter2));

        let router = Router::new();
        let _ = manager.configure_routes(router).await.unwrap();

        assert!(after_route_called1.load(Ordering::SeqCst));
        assert!(after_route_called2.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_adapter_manager() {
        let ctx = Context::default(); // You'll need to implement this
        let mut adapter_manager = AdapterManager::new(ctx);

        let app = Router::new();
        // Initialize adapters
        adapter_manager.register(Box::new(MockAdapter::new(
            "signal-test",
            AdapterPriority::Normal,
        )));

        adapter_manager.init_all().await.unwrap();
        let _ = adapter_manager.before_run().await.unwrap();
        let _ = adapter_manager.configure_routes(app.clone()).await.unwrap();

        let mut rx = adapter_manager.shutdown_signal();
        tokio::spawn(async move {
            adapter_manager.stop_all().await.unwrap();
        });

        assert!(rx.recv().await.is_ok());
    }
}
