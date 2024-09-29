use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Page<T> {
    #[serde(rename(serialize = "results"))]
    pub results: T,

    #[serde(rename(serialize = "pagination"))]
    pub meta: Meta,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Meta {
    #[serde(rename(serialize = "page"))]
    pub page: u64,
    #[serde(rename(serialize = "page_size"))]
    pub page_size: u64,
    #[serde(rename(serialize = "total_pages"))]
    pub total_pages: u64,
}

impl<T> Page<T> {
    #[must_use]
    pub const fn new(results: T, meta: Meta) -> Self {
        Self { results, meta }
    }
}
