use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use post_archiver::{
    manager::PostArchiverManager, FileMeta, FileMetaId, PostId, POSTS_PRE_CHUNK,
};
use rusqlite::Row;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct Pagination {
    pub limit: Option<u32>,
    pub page: Option<u32>,
}

impl Pagination {
    pub fn limit(&self) -> u32 {
        self.limit.unwrap_or(20)
    }

    pub fn page(&self) -> u32 {
        self.page.unwrap_or(0)
    }

    pub fn to_sql(&self) -> String {
        let limit = self.limit();
        let page = self.page() * limit;
        format!("LIMIT {} OFFSET {}", limit, page)
    }
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PostMiniJson {
    pub id: PostId,
    pub title: String,
    pub thumb: Option<String>,
    pub updated: DateTime<Utc>,
}

impl PostMiniJson {
    pub fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        let id = row.get("id")?;
        let filename: Option<String> = row.get("thumb")?;
        Ok(PostMiniJson {
            id,
            title: row.get("title")?,
            updated: row.get("updated")?,
            thumb: filename.map(|f| into_thumb_url(id, f)),
        })
    }
}

pub fn get_file_metas(
    manager: &PostArchiverManager,
    file_ids: HashSet<FileMetaId>,
) -> Result<HashMap<FileMetaId, FileMeta>, rusqlite::Error> {
    let mut stmt = manager
        .conn()
        .prepare_cached("SELECT * FROM file_metas WHERE id IN (SELECT value FROM json_each(?))")?;

    let file_metas = stmt.query_map([serde_json::to_string(&file_ids).unwrap()], |row| {
        FileMeta::from_row(row).map(|meta| (meta.id, meta))
    })?;

    file_metas.collect()
}

pub fn into_thumb_url(post: PostId, filename: String) -> String {
    format!(
        "{}/{}/{}",
        *post / POSTS_PRE_CHUNK,
        *post % POSTS_PRE_CHUNK,
        filename
    )
}

#[derive(Debug, Clone, Serialize, TS)]
pub struct WithThumb<T> {
    #[serde(flatten)]
    pub category: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb: Option<FileMeta>,
}
