use chrono::{DateTime, Utc};
use post_archiver::{AuthorId, CollectionId, FileMetaId, PlatformId, PostId, TagId};
use rusqlite::Row;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::relation::RequireRelations;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub fn params(&self) -> [u32; 2] {
        let limit = self.limit();
        let page = self.page() * limit;
        [limit, page]
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct PostPreview {
    pub id: PostId,
    pub title: String,
    pub thumb: Option<FileMetaId>,
    pub updated: DateTime<Utc>,
}

impl PostPreview {
    pub fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: row.get("id")?,
            title: row.get("title")?,
            thumb: row.get("thumb")?,
            updated: row.get("updated")?,
        })
    }
}

impl RequireRelations for PostPreview {
    fn file_metas(&self) -> Vec<FileMetaId> {
        self.thumb.into_iter().collect()
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct ListResponse<T> {
    pub list: Vec<T>,
    pub total: usize,
}

impl<T: RequireRelations + TS> RequireRelations for ListResponse<T> {
    fn authors(&self) -> Vec<AuthorId> {
        self.list.authors()
    }
    fn collections(&self) -> Vec<CollectionId> {
        self.list.collections()
    }
    fn platforms(&self) -> Vec<PlatformId> {
        self.list.platforms()
    }
    fn tags(&self) -> Vec<TagId> {
        self.list.tags()
    }
    fn file_metas(&self) -> Vec<FileMetaId> {
        self.list.file_metas()
    }
}
