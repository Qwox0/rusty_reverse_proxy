use axum::http::{Extensions, HeaderMap, HeaderValue, Method, Uri, Version};

pub struct RequestParts {
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
    pub headers: HeaderMap<HeaderValue>,
    pub extensions: Extensions,
}

pub trait AsReqParts {
    type Parts;

    fn as_parts(&self) -> &Self::Parts;
}
