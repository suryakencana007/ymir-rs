use std::{
    convert::Infallible,
    task::{Context, Poll},
};

use axum::{extract::FromRequestParts, Error};
use http::{request::Parts, Request};
use tower::Service;

#[derive(Clone, Copy, Debug)]
pub struct InjectState<S, T> {
    pub inner: S,
    pub value: T,
}

impl<ResBody, S, T> Service<Request<ResBody>> for InjectState<S, T>
where
    S: Service<Request<ResBody>>,
    T: Clone + Send + Sync + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ResBody>) -> Self::Future {
        req.extensions_mut().insert(self.value.clone());
        self.inner.call(req)
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[must_use]
pub struct Inject<T>(pub T);

impl<T, S> FromRequestParts<S> for Inject<T>
where
    T: Clone + Send + Sync + 'static,
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(req: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let value = req
            .extensions
            .get::<T>()
            .ok_or_else(|| {
                Error::new(format!("Extension of type `{}` was not found. Perhaps you forgot to add it? See `ymir::Extension`.", std::any::type_name::<T>()))
            }).cloned().unwrap();

        Ok(Inject(value))
    }
}

impl<S, T> tower_layer::Layer<S> for Inject<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Service = InjectState<S, T>;

    fn layer(&self, inner: S) -> Self::Service {
        InjectState {
            inner,
            value: self.0.clone(),
        }
    }
}
