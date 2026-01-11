use axum::Router;
mod auth;
mod challenges;
mod export;
mod attachments;

pub fn router() -> Router<crate::State> {
    Router::new()
        .nest("/challs", challenges::router())
        .nest("/auth", auth::router())
        .nest("/export", export::router())
        .nest("/attachments", attachments::router())
}
