use axum::{extract::State as StateE, routing::post, Json, Router, http::StatusCode};
use chrono::Duration;
use nanoid::nanoid;
use serde::Deserialize;

use crate::{extractors::Admin, jwt::generate_jwt, Result, State};
use crate::api::Team;

#[derive(Deserialize)]
struct ResendToken {
    email: String,
}

#[derive(Deserialize)]
struct CreateTeamRequest {
    name: String,
    email: String,
    division: Option<String>,
}

#[derive(Deserialize)]
struct ImpersonateTeamRequest {
    name: Option<String>,
    email: Option<String>,
    token_expiration: Option<String>,
}

async fn resend_token(
    StateE(state): StateE<State>,
    _: Admin,
    Json(payload): Json<ResendToken>,
) -> Result<()> {
    let team_partial = sqlx::query!(
        "SELECT public_id, name FROM teams WHERE email = $1",
        payload.email,
    )
    .fetch_one(&state.db)
    .await?;

    let jwt = generate_jwt(
        &state.config.jwt_keys,
        &team_partial.public_id,
        Duration::days(30),
    )?;

    state
        .email
        .send_resend_token_email(&state.event, &payload.email, &team_partial.name, &jwt)
        .await?;

    Ok(())
}

async fn create_team(
    StateE(state): StateE<State>,
    _: Admin,
    Json(payload): Json<CreateTeamRequest>,
) -> Result<Json<Team>> {
    let team = sqlx::query_as!(
        Team, // TODO: fix and include more fields
        "INSERT INTO teams (public_id, name, email, division) VALUES ($1, $2, $3, $4) RETURNING *",
        nanoid!(),
        payload.name,
        payload.email,
        payload.division,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(team))
}

async fn impersonate_team(
    StateE(state): StateE<State>,
    _: Admin,
    Json(payload): Json<ImpersonateTeamRequest>,
) -> Result<(StatusCode, Json<String>)> {
    let team_public_id = if let Some(name) = payload.name {
        sqlx::query!("SELECT public_id FROM teams WHERE name = $1", name)
            .fetch_one(&state.db)
            .await?.public_id
    } else if let Some(email) = payload.email {
        sqlx::query!("SELECT public_id FROM teams WHERE email = $1", email)
            .fetch_one(&state.db)
            .await?.public_id
    } else {
        return Ok((StatusCode::BAD_REQUEST, Json("invalid request".to_string())));
    };

    let expiration = if let Some(exp_str) = payload.token_expiration {
        Duration::days(exp_str.parse().unwrap_or(30))
    } else {
        Duration::days(30)
    };

    let jwt = generate_jwt(&state.config.jwt_keys, &team_public_id, expiration)?;
    
    Ok((StatusCode::OK, Json(jwt)))
}

pub fn router() -> Router<crate::State> {
    Router::new()
        .route("/resend_token", post(resend_token))
        .route("/create_team", post(create_team))
        .route("/impersonate_team", post(impersonate_team))
}
