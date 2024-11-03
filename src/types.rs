use std::borrow::Cow;

use serde::Serialize;
use utoipa::{PartialSchema, ToSchema};

#[derive(Serialize)]
pub struct Ulid(ulid::Ulid);

impl Ulid {
    pub fn new() -> Self {
        Self(ulid::Ulid::new())
    }
}

impl PartialSchema for Ulid {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::schema::Object::builder()
            .schema_type(utoipa::openapi::schema::SchemaType::AnyValue)
            .into()
    }
}

impl ToSchema for Ulid {
    fn name() -> std::borrow::Cow<'static, str> {
        Cow::Borrowed("UlidSchema")
    }
}
