use axum::Router;

pub mod updates;

pub trait EndpointModule {
    fn create_router() -> Router;
}
