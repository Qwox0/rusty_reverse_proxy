use axum::extract::{FromRequestParts, Request};
use std::future::Future;

pub trait RequestExtract: Sized {
    fn extract<T: FromRequestParts<()>>(
        self,
    ) -> impl Future<Output = Result<(Self, T), T::Rejection>>;
}

impl RequestExtract for Request {
    fn extract<T: FromRequestParts<()>>(
        self,
    ) -> impl Future<Output = Result<(Self, T), T::Rejection>> {
        async {
            let (mut parts, body) = self.into_parts();
            let t = T::from_request_parts(&mut parts, &()).await;
            t.map(|t| (Request::from_parts(parts, body), t))
        }
    }
}
