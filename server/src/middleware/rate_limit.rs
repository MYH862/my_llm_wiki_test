use axum::{
    extract::ConnectInfo,
    http::{Request, StatusCode},
    response::Response,
};
use governor::{
    clock::DefaultClock,
    middleware::NoOpMiddleware,
    state::keyed::DashMapStateStore,
    Quota, RateLimiter,
};
use std::{
    net::SocketAddr,
    num::NonZeroU32,
    sync::Arc,
    task::{Context, Poll},
};
use tower::{Layer, Service};

pub type SharedRateLimiter = Arc<RateLimiter<SocketAddr, DashMapStateStore<SocketAddr>, DefaultClock, NoOpMiddleware>>;

pub fn create_rate_limiter(requests_per_second: u32) -> SharedRateLimiter {
    let quota = Quota::per_second(NonZeroU32::new(requests_per_second).unwrap_or(NonZeroU32::new(10).unwrap()));
    Arc::new(RateLimiter::dashmap(quota))
}

#[derive(Clone)]
pub struct RateLimitMiddleware {
    limiter: SharedRateLimiter,
}

impl RateLimitMiddleware {
    pub fn new(limiter: SharedRateLimiter) -> Self {
        Self { limiter }
    }
}

impl<S> Layer<S> for RateLimitMiddleware {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> <Self as Layer<S>>::Service {
        RateLimitService {
            inner,
            limiter: self.limiter.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RateLimitService<S> {
    inner: S,
    limiter: SharedRateLimiter,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for RateLimitService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<axum::BoxError>,
    ReqBody: Send + 'static,
    ResBody: Default + Send + 'static,
{
    type Response = Response<ResBody>;
    type Error = axum::BoxError;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let limiter = self.limiter.clone();
        let connect_info = req.extensions().get::<ConnectInfo<SocketAddr>>().cloned();
        let future = self.inner.call(req);

        Box::pin(async move {
            if let Some(ConnectInfo(addr)) = connect_info {
                match limiter.check_key(&addr) {
                    Ok(_) => future.await.map_err(Into::into),
                    Err(_) => Ok(Response::builder()
                        .status(StatusCode::TOO_MANY_REQUESTS)
                        .body(ResBody::default())
                        .unwrap()),
                }
            } else {
                future.await.map_err(Into::into)
            }
        })
    }
}

pub async fn rate_limit_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    limiter: axum::extract::State<SharedRateLimiter>,
) -> Result<(), StatusCode> {
    match limiter.check_key(&addr) {
        Ok(_) => Ok(()),
        Err(_) => Err(StatusCode::TOO_MANY_REQUESTS),
    }
}
