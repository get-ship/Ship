use axum::Router;

pub fn router() -> Router {
    crate::build_router()
}
