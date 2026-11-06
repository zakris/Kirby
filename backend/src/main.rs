use std::time::Duration;

use axum::{
    extract::{Path, Query, State},
    http::{HeaderValue, Method, StatusCode},
    routing::{get, patch},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

#[derive(Serialize, sqlx::FromRow)]
struct Task {
    id: Uuid,
    title: String,
    /// Day the task belongs to, `YYYY-MM-DD`.
    date: String,
    done: bool,
}

#[derive(Deserialize)]
struct NewTask {
    title: String,
    date: String,
}

#[derive(Deserialize)]
struct UpdateTask {
    title: Option<String>,
    done: Option<bool>,
}

#[derive(Deserialize)]
struct ListQuery {
    date: Option<String>,
}

const COLS: &str = "id, title, to_char(date, 'YYYY-MM-DD') AS date, done";

fn db_err(e: sqlx::Error) -> StatusCode {
    // Invalid date strings surface as a database error; treat them as bad input.
    if let sqlx::Error::Database(d) = &e {
        if d.code().as_deref() == Some("22007") || d.code().as_deref() == Some("22008") {
            return StatusCode::BAD_REQUEST;
        }
    }
    eprintln!("db error: {e}");
    StatusCode::INTERNAL_SERVER_ERROR
}

async fn list(State(db): State<PgPool>, Query(q): Query<ListQuery>) -> Result<Json<Vec<Task>>, StatusCode> {
    let sql = format!(
        "SELECT {COLS} FROM tasks WHERE ($1::text IS NULL OR date = $1::date) ORDER BY date, created_at"
    );
    let tasks = sqlx::query_as::<_, Task>(&sql).bind(q.date).fetch_all(&db).await.map_err(db_err)?;
    Ok(Json(tasks))
}

async fn create(State(db): State<PgPool>, Json(n): Json<NewTask>) -> Result<(StatusCode, Json<Task>), StatusCode> {
    let sql = format!("INSERT INTO tasks (id, title, date) VALUES ($1, $2, $3::date) RETURNING {COLS}");
    let task = sqlx::query_as::<_, Task>(&sql)
        .bind(Uuid::new_v4())
        .bind(n.title)
        .bind(n.date)
        .fetch_one(&db)
        .await
        .map_err(db_err)?;
    Ok((StatusCode::CREATED, Json(task)))
}

async fn update(State(db): State<PgPool>, Path(id): Path<Uuid>, Json(u): Json<UpdateTask>) -> Result<Json<Task>, StatusCode> {
    let sql = format!(
        "UPDATE tasks SET title = COALESCE($2, title), done = COALESCE($3, done) WHERE id = $1 RETURNING {COLS}"
    );
    sqlx::query_as::<_, Task>(&sql)
        .bind(id)
        .bind(u.title)
        .bind(u.done)
        .fetch_optional(&db)
        .await
        .map_err(db_err)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn delete(State(db): State<PgPool>, Path(id): Path<Uuid>) -> Result<StatusCode, StatusCode> {
    let r = sqlx::query("DELETE FROM tasks WHERE id = $1").bind(id).execute(&db).await.map_err(db_err)?;
    Ok(if r.rows_affected() > 0 { StatusCode::NO_CONTENT } else { StatusCode::NOT_FOUND })
}

fn cors() -> CorsLayer {
    match std::env::var("KIRBY_ALLOWED_ORIGIN") {
        Ok(origin) => CorsLayer::new()
            .allow_origin(origin.parse::<HeaderValue>().expect("invalid KIRBY_ALLOWED_ORIGIN"))
            .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
            .allow_headers(Any),
        // Local development: no origin configured, allow everything.
        Err(_) => CorsLayer::permissive(),
    }
}

/// Read a connection string from the environment, ignoring empty values and
/// stripping stray whitespace/quotes (easy to introduce when pasting secrets into a dashboard).
fn env_url(name: &str) -> Option<String> {
    let v = std::env::var(name).ok()?;
    let v = v.trim().trim_matches(|c| c == '"' || c == '\'').trim();
    (!v.is_empty()).then(|| v.to_string())
}

fn check_url(name: &str, url: &str) {
    if !(url.starts_with("postgres://") || url.starts_with("postgresql://")) {
        panic!("{name} must start with postgres:// or postgresql:// (check the value set in your host's env settings)");
    }
}

#[tokio::main]
async fn main() {
    // Pooled URL (PgBouncer) for request traffic.
    let url = env_url("DATABASE_URL").expect("DATABASE_URL must be set");
    check_url("DATABASE_URL", &url);
    // Migrations take session-level advisory locks, so they need a direct (non-pooled) connection.
    let direct_url = env_url("DATABASE_URL_UNPOOLED").unwrap_or_else(|| url.clone());
    check_url("DATABASE_URL_UNPOOLED", &direct_url);

    let migrator = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(30)) // Neon may need a moment to wake from idle
        .connect(&direct_url)
        .await
        .expect("failed to connect to database (direct)");
    sqlx::migrate!().run(&migrator).await.expect("migrations failed");
    migrator.close().await;

    let db = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&url)
        .await
        .expect("failed to connect to database");

    let app = Router::new()
        .route("/tasks", get(list).post(create))
        .route("/tasks/{id}", patch(update).delete(delete))
        .route("/health", get(|| async { "ok" }))
        .layer(cors())
        .with_state(db);

    // Render provides PORT; KIRBY_ADDR overrides for local use.
    let addr = std::env::var("KIRBY_ADDR").unwrap_or_else(|_| {
        format!("0.0.0.0:{}", std::env::var("PORT").unwrap_or_else(|_| "8080".into()))
    });
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("kirby-backend listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}
