use axum::{extract::State, http::StatusCode, Json};
use post_archiver::utils::VERSION;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SummaryResponse {
    version: String,
    post_archiver_version: String,
    tags: u32,
    authors: u32,
    collections: u32,
    platforms: u32,
    posts: u32,
    files: u32,
}

pub async fn get_summary_api(
    State(state): State<AppState>,
) -> Result<Json<SummaryResponse>, StatusCode> {
    let manager = state.manager();
    let conn = manager.conn();

    let post_archiver_version: String = conn
        .query_row("SELECT version FROM post_archiver_meta", [], |row| {
            row.get(0)
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let platforms: u32 = conn
        .query_row("SELECT COUNT() FROM platforms", [], |row| row.get(0))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tags: u32 = conn
        .query_row("SELECT COUNT() FROM tags", [], |row| row.get(0))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let authors: u32 = conn
        .query_row("SELECT COUNT() FROM authors", [], |row| row.get(0))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let collections: u32 = conn
        .query_row("SELECT COUNT() FROM collections", [], |row| row.get(0))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let posts: u32 = conn
        .query_row("SELECT COUNT() FROM posts", [], |row| row.get(0))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let files: u32 = conn
        .query_row("SELECT COUNT() FROM file_metas", [], |row| row.get(0))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SummaryResponse {
        version: VERSION.to_string(),
        post_archiver_version,
        platforms,
        collections,
        authors,
        tags,
        posts,
        files,
    }))
}
