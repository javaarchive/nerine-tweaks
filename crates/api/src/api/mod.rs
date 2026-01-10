use axum::Router;

mod auth;
mod challenges;
mod leaderboard;
mod profile;

// TODO: is this really how we want to import the team struct into admin?
pub use auth::Team;

pub fn router() -> Router<crate::State> {
    Router::new()
        .nest("/admin", crate::admin::router())
        .nest("/auth", auth::router())
        .nest("/challs", challenges::router())
        .nest("/event", crate::event::router())
        .nest("/profile", profile::router())
        .nest("/leaderboard", leaderboard::router())
}
