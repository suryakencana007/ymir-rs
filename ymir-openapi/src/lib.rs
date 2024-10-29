pub mod prelude;
pub mod router;

use std::borrow::Cow;

use axum::{
    response::Html,
    routing::{self, MethodFilter},
    Router,
};
use serde::Serialize;
use serde_json::Value;
use utoipa::openapi::{HttpMethod, OpenApi};

const DEFAULT_HTML: &str = include_str!("../assets/swagger.html");

#[derive(Clone)]
pub struct Swagger<S: Spec> {
    #[allow(unused)]
    url: Cow<'static, str>,
    html: Cow<'static, str>,
    openapi: S,
}

impl<S: Spec> Swagger<S> {
    pub fn new(openapi: S) -> Self {
        Self {
            url: Cow::Borrowed(""),
            html: Cow::Borrowed(DEFAULT_HTML),
            openapi,
        }
    }

    pub fn to_html(&self) -> String {
        self.html.replace(
            "$spec",
            &serde_json::to_string(&self.openapi).expect(
                "Invalid OpenAPI spec, expected OpenApi, String, &str or serde_json::Value",
            ),
        )
    }
}

pub trait Spec: Serialize {}

impl Spec for OpenApi {}

impl Spec for String {}

impl Spec for &str {}

impl Spec for Value {}

pub trait Servable<S>
where
    S: Spec,
{
    /// Construct a new [`Servable`] instance of _`openapi`_ with given _`url`_.
    ///
    /// * **url** Must point to location where the [`Servable`] is served.
    /// * **openapi** Is [`Spec`] that is served via this [`Servable`] from the _**url**_.
    fn with_url<U: Into<Cow<'static, str>>>(url: U, openapi: S) -> Self;
}

// #[cfg(any(feature = "axum"))]
impl<S: Spec> Servable<S> for Swagger<S> {
    fn with_url<U: Into<Cow<'static, str>>>(url: U, openapi: S) -> Self {
        Self {
            url: url.into(),
            openapi,
            html: Cow::Borrowed(DEFAULT_HTML),
        }
    }
}

impl<S: Spec, R> From<Swagger<S>> for Router<R>
where
    R: Clone + Send + Sync + 'static,
{
    fn from(value: Swagger<S>) -> Self {
        let html = value.to_html();
        Router::<R>::new().route(&value.url.as_ref(), routing::get(|| async { Html(html) }))
    }
}

/// Extends [`utoipa::openapi::path::PathItem`] by providing conversion methods to convert this
/// path item type to a [`axum::routing::MethodFilter`].
pub trait PathItemExt {
    /// Convert this path item type to a [`axum::routing::MethodFilter`].
    ///
    /// Method filter is used with handler registration on [`axum::routing::MethodRouter`].
    fn to_method_filter(&self) -> MethodFilter;
}

impl PathItemExt for HttpMethod {
    fn to_method_filter(&self) -> MethodFilter {
        match self {
            HttpMethod::Get => MethodFilter::GET,
            HttpMethod::Put => MethodFilter::PUT,
            HttpMethod::Post => MethodFilter::POST,
            HttpMethod::Head => MethodFilter::HEAD,
            HttpMethod::Patch => MethodFilter::PATCH,
            HttpMethod::Trace => MethodFilter::TRACE,
            HttpMethod::Delete => MethodFilter::DELETE,
            HttpMethod::Options => MethodFilter::OPTIONS,
        }
    }
}

/// re-export paste so users do not need to add the dependency.
#[doc(hidden)]
pub use paste::paste;

#[macro_export]
macro_rules! routes {
    ( $handler:path $(, $tail:path)* ) => {
        {
            use $crate::PathItemExt;
            let mut paths = utoipa::openapi::path::Paths::new();
            let mut schemas = Vec::<(String, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>)>::new();
            let (path, item, types) = $crate::routes!(@resolve_types $handler : schemas);
            #[allow(unused_mut)]
            let mut method_router = types.iter().by_ref().fold(axum::routing::MethodRouter::new(), |router, path_type| {
                router.on(path_type.to_method_filter(), $handler)
            });
            paths.add_path_operation(&path, types, item);
            $( method_router = $crate::routes!( schemas: method_router: paths: $tail ); )*
            (schemas, paths, method_router)
        }
    };
    ( $schemas:tt: $router:ident: $paths:ident: $handler:path $(, $tail:tt)* ) => {
        {
            let (path, item, types) = $crate::routes!(@resolve_types $handler : $schemas);
            let router = types.iter().by_ref().fold($router, |router, path_type| {
                router.on(path_type.to_method_filter(), $handler)
            });
            $paths.add_path_operation(&path, types, item);
            router
        }
    };
    ( @resolve_types $handler:path : $schemas:tt ) => {
        {
            $crate::paste! {
                let path = $crate::routes!( @path [path()] of $handler );
                let mut operation = $crate::routes!( @path [operation()] of $handler );
                let types = $crate::routes!( @path [methods()] of $handler );
                let tags = $crate::routes!( @path [tags()] of $handler );
                $crate::routes!( @path [schemas(&mut $schemas)] of $handler );
                if !tags.is_empty() {
                    let operation_tags = operation.tags.get_or_insert(Vec::new());
                    operation_tags.extend(tags.iter().map(ToString::to_string));
                }
                (path, operation, types)
            }
        }
    };
    ( @path $op:tt of $part:ident $( :: $tt:tt )* ) => {
        $crate::routes!( $op : [ $part $( $tt )*] )
    };
    ( $op:tt : [ $first:tt $( $rest:tt )* ] $( $rev:tt )* ) => {
        $crate::routes!( $op : [ $( $rest )* ] $first $( $rev)* )
    };
    ( $op:tt : [] $first:tt $( $rest:tt )* ) => {
        $crate::routes!( @inverse $op : $first $( $rest )* )
    };
    ( @inverse $op:tt : $tt:tt $( $rest:tt )* ) => {
        $crate::routes!( @rev $op : $tt [$($rest)*] )
    };
    ( @rev $op:tt : $tt:tt [ $first:tt $( $rest:tt)* ] $( $reversed:tt )* ) => {
        $crate::routes!( @rev $op : $tt [ $( $rest )* ] $first $( $reversed )* )
    };
    ( @rev [$op:ident $( $args:tt )* ] : $handler:tt [] $($tt:tt)* ) => {
        {
            #[allow(unused_imports)]
            use utoipa::{Path, __dev::{Tags, SchemaReferences}};
            $crate::paste! {
                $( $tt :: )* [<__path_ $handler>]::$op $( $args )*
            }
        }
    };
    ( ) => {};
}
