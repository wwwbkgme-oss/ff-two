use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Sse,
    Json,
};
use futures::stream::Stream;
use tokio_stream::{wrappers::BroadcastStream, StreamExt as _};
use uuid::Uuid;

use types::WorldSnapshot;

use crate::errors::ApiResult;
use crate::state::AppState;

pub async fn get_state(State(s): State<AppState>, Path(pid): Path<Uuid>) -> ApiResult<Json<serde_json::Value>> {
    s.store.get_project(pid).await?;
    let chunks = s.world.all_chunks();
    let (cc, bc) = (chunks.len(), chunks.iter().map(|c| c.blocks.len()).sum::<usize>());
    Ok(Json(serde_json::json!({ "project_id": pid, "chunk_count": cc, "block_count": bc, "chunks": chunks })))
}

pub async fn stream_events(
    State(s): State<AppState>,
    Path(_pid): Path<Uuid>,
) -> Sse<impl Stream<Item = Result<axum::response::sse::Event, std::convert::Infallible>>> {
    let stream = BroadcastStream::new(s.world.subscribe()).filter_map(|msg| {
        let data = serde_json::to_string(&msg.ok()?).ok()?;
        Some(Ok(axum::response::sse::Event::default().data(data)))
    });
    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}

pub async fn get_chunk(State(s): State<AppState>, Path((pid, cx, cz)): Path<(Uuid, i32, i32)>) -> ApiResult<Json<serde_json::Value>> {
    s.store.get_project(pid).await?;
    Ok(Json(serde_json::json!({ "chunk": s.world.get_chunk(cx, 0, cz) })))
}

pub async fn list_structures(State(s): State<AppState>, Path(pid): Path<Uuid>) -> ApiResult<Json<serde_json::Value>> {
    s.store.get_project(pid).await?;
    let chunks = s.world.all_chunks();
    let blocks: Vec<_> = chunks.iter().flat_map(|c| c.blocks.iter()).collect();
    let count = blocks.len();
    Ok(Json(serde_json::json!({ "project_id": pid, "structures": blocks, "count": count })))
}

pub async fn list_snapshots(State(s): State<AppState>, Path(pid): Path<Uuid>) -> ApiResult<Json<serde_json::Value>> {
    let snaps = s.store.list_snapshots(pid).await?;
    let count = snaps.len();
    Ok(Json(serde_json::json!({ "snapshots": snaps, "count": count })))
}

pub async fn take_snapshot(State(s): State<AppState>, Path(pid): Path<Uuid>, Json(b): Json<serde_json::Value>) -> ApiResult<(StatusCode, Json<WorldSnapshot>)> {
    s.store.get_project(pid).await?;
    let snap = s.world.snapshot(pid, b.get("message").and_then(|v| v.as_str()).unwrap_or("manual snapshot"));
    Ok((StatusCode::CREATED, Json(s.store.save_snapshot(snap).await?)))
}
