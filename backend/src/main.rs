use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, patch},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
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

type Store = Arc<Mutex<HashMap<Uuid, Task>>>;

async fn list(State(s): State<Store>, Query(q): Query<ListQuery>) -> Json<Vec<Task>> {
    let store = s.lock().await;
    let mut tasks: Vec<Task> = store
        .values()
        .filter(|t| q.date.as_ref().is_none_or(|d| &t.date == d))
        .cloned()
        .collect();
    tasks.sort_by(|a, b| a.date.cmp(&b.date).then(a.title.cmp(&b.title)));
    Json(tasks)
}

async fn create(State(s): State<Store>, Json(n): Json<NewTask>) -> (StatusCode, Json<Task>) {
    let task = Task { id: Uuid::new_v4(), title: n.title, date: n.date, done: false };
    s.lock().await.insert(task.id, task.clone());
    (StatusCode::CREATED, Json(task))
}

async fn update(
    State(s): State<Store>,
    Path(id): Path<Uuid>,
    Json(u): Json<UpdateTask>,
) -> Result<Json<Task>, StatusCode> {
    let mut store = s.lock().await;
    let task = store.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    if let Some(t) = u.title { task.title = t; }
    if let Some(d) = u.done { task.done = d; }
    Ok(Json(task.clone()))
}

async fn delete(State(s): State<Store>, Path(id): Path<Uuid>) -> StatusCode {
    if s.lock().await.remove(&id).is_some() { StatusCode::NO_CONTENT } else { StatusCode::NOT_FOUND }
}

#[tokio::main]
async fn main() {
    let store: Store = Arc::default();
    let app = Router::new()
        .route("/tasks", get(list).post(create))
        .route("/tasks/{id}", patch(update).delete(delete))
        .route("/health", get(|| async { "ok" }))
        .layer(CorsLayer::permissive()) // tighten to the Pages origin before deploying
        .with_state(store);

    let addr = std::env::var("KIRBY_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("kirby-backend listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}
