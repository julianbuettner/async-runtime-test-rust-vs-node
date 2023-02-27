use async_trait::async_trait;
use futures::TryStreamExt;
use std::{net::SocketAddr, time::Duration};

use axum::{
    extract::{FromRef, FromRequestParts},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Serialize;
use sqlx::{
    postgres::{PgPoolOptions, PgRow},
    PgPool,
};

const HASH_COST: usize = 100_000;

fn expensive_hash(input: i32) -> i32 {
    (1..HASH_COST).fold(123, |acc, x| (acc + x as i32 + input) % 99999)
}

#[derive(Serialize)]
pub struct UserInfo {
    pub id: i32,
    pub name: String,
    pub expensive_hash: i32,
}

async fn list_users(
    DatabaseConnection(mut conn): DatabaseConnection,
) -> (StatusCode, Json<Vec<UserInfo>>) {
    let users_db = sqlx::query!("SELECT id, name, hash_basis FROM users")
        .fetch_all(&mut conn)
        .await
        .unwrap();

    let users: Vec<UserInfo> = users_db
        .into_iter()
        .map(|rec| UserInfo {
            id: rec.id,
            name: rec.name.unwrap_or_else(|| String::new()),
            expensive_hash: rec.hash_basis.map(|x| expensive_hash(x)).unwrap_or(0),
        })
        .collect();

    (StatusCode::OK, Json(users))
}

#[tokio::main]
async fn main() {
    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://async:async@localhost".to_string());
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&db_connection_str)
        .await
        .expect("Could not connect to Database");

    let app = Router::new()
        .route("/list", get(list_users))
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Running now...");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    todo!()
}

struct DatabaseConnection(sqlx::pool::PoolConnection<sqlx::Postgres>);

#[async_trait]
impl<S> FromRequestParts<S> for DatabaseConnection
where
    PgPool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let pool = PgPool::from_ref(state);

        let conn = pool.acquire().await.map_err(internal_error)?;

        Ok(Self(conn))
    }
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
