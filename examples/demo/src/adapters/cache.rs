use std::convert::Infallible;

use axum::{extract::FromRequestParts, http::request::Parts, Error};
use ymir::state::InjectState;

#[derive(Debug, Clone, Copy, Default)]
#[must_use]
pub struct Cache<T>(pub T);

impl<T, S> FromRequestParts<S> for Cache<T>
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

        Ok(Cache(value))
    }
}

impl<S, T> tower_layer::Layer<S> for Cache<T>
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
