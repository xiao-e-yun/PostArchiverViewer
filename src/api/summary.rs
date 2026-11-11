use axum::{Json, extract::State};
use post_archiver::{
    query::{PostArchiverQueryExt, Query},
    utils::VERSION,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{AppState, error::ApiResult};

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

pub async fn get_summary_api(State(state): State<AppState>) -> ApiResult<Json<SummaryResponse>> {
    let manager = state.manager();

    // The lib has no public getter for the archive version yet.
    let post_archiver_version: String =
        manager
            .conn
            .query_row("SELECT version FROM post_archiver_meta", [], |row| {
                row.get(0)
            })?;

    Ok(Json(SummaryResponse {
        version: VERSION.to_string(),
        post_archiver_version,
        platforms: manager.platforms().query().count()? as u32,
        collections: manager.collections().query().count()? as u32,
        authors: manager.authors().query().count()? as u32,
        tags: manager.tags().query().count()? as u32,
        posts: manager.posts().query().count()? as u32,
        files: manager.file_metas().query().count()? as u32,
    }))
}
