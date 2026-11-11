use std::fmt::Debug;

use post_archiver::{
    FileMeta, FileMetaId, Platform, PlatformId, manager::PostArchiverManager,
    query::PostArchiverQueryExt,
};
use serde::Serialize;
use ts_rs::TS;

use super::error::ApiResult;

/// A response plus the platforms and file metas it refers to by id,
/// so the frontend can resolve them without extra requests.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct WithRelations<T: Debug> {
    #[serde(flatten)]
    pub inner: T,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<Platform>>", optional)]
    pub platforms: Vec<Platform>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<FileMeta>>", optional)]
    pub file_metas: Vec<FileMeta>,
}

impl<T: Debug> WithRelations<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            platforms: Vec::new(),
            file_metas: Vec::new(),
        }
    }

    pub fn platforms(
        mut self,
        manager: &PostArchiverManager,
        ids: impl IntoIterator<Item = PlatformId>,
    ) -> ApiResult<Self> {
        self.platforms = manager.platforms().get_many(ids)?;
        Ok(self)
    }

    pub fn file_metas(
        mut self,
        manager: &PostArchiverManager,
        ids: impl IntoIterator<Item = FileMetaId>,
    ) -> ApiResult<Self> {
        self.file_metas = manager.file_metas().get_many(ids)?;
        Ok(self)
    }
}
