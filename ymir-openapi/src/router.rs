use std::{borrow::Cow, convert::Infallible};

use axum::{routing::MethodRouter, Router};
use utoipa::{openapi, OpenApi};

use crate::{Servable, Swagger};

const DEFAULT_OPENAPI: &str = "/swagger-ui";

#[inline]
fn colonized_params<S: AsRef<str>>(path: S) -> String
where
    String: From<S>,
{
    String::from(path).replace('}', "").replace('{', ":")
}

pub type OpenApiMethod<S = (), E = Infallible> = (
    Vec<(
        String,
        utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
    )>,
    utoipa::openapi::path::Paths,
    axum::routing::MethodRouter<S, E>,
);

#[derive(Clone)]
pub struct RouterDoc<S = ()>(Router<S>, utoipa::openapi::OpenApi, Cow<'static, str>);

impl<S> RouterDoc<S>
where
    S: Send + Sync + Clone + 'static,
{
    pub fn new() -> RouterDoc<S> {
        use utoipa::OpenApi;
        #[derive(OpenApi)]
        pub struct ApiDoc;

        Self(
            Router::new(),
            ApiDoc::openapi(),
            Cow::Borrowed(DEFAULT_OPENAPI),
        )
    }

    pub fn build_doc<U: Into<Cow<'static, str>>>(
        self,
        path: U,
        build_doc: fn(oa: openapi::OpenApi) -> openapi::OpenApi,
    ) -> Self {
        Self(self.0, build_doc(self.1), path.into())
    }

    pub fn routes(mut self, (schemas, mut paths, method_router): OpenApiMethod<S>) -> Self {
        let router = if paths.paths.len() == 1 {
            let first_entry = &paths.paths.first_entry();
            let path = first_entry.as_ref().map(|path| path.key());
            let Some(path) = path else {
                unreachable!("Whoopsie, I thought there was one Path entry");
            };
            let path = if path.is_empty() { "/" } else { path };

            self.0.route(&colonized_params(path), method_router)
        } else {
            paths.paths.iter().fold(self.0, |this, (path, _)| {
                let path = if path.is_empty() { "/" } else { path };
                this.route(&colonized_params(path), method_router.clone())
            })
        };

        // add current paths to the OpenApi
        self.1.paths.paths.extend(paths.paths.clone());
        let components = self
            .1
            .components
            .get_or_insert(utoipa::openapi::Components::new());
        components.schemas.extend(schemas);

        Self(router, self.1, self.2)
    }

    /// Pass through method for [`axum::Router<S>::route`].
    pub fn route(self, path: &str, method_router: MethodRouter<S>) -> Router<S> {
        self.0.clone().merge(Self(
            Router::new().route(path, method_router),
            self.1,
            self.2,
        ))
    }
}

impl<S> From<RouterDoc<S>> for Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn from(value: RouterDoc<S>) -> Self {
        value.0.merge(Swagger::with_url(value.2, value.1))
    }
}
