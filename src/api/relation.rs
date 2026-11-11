use std::fmt::Debug;

use post_archiver::{
    FileMeta, FileMetaId, Platform, PlatformId, manager::PostArchiverManager,
    query::PostArchiverQueryExt,
};
use serde::Serialize;
use ts_rs::TS;

use super::error::ApiResult;

/// Rows of `$repo` with the given ids, e.g. `by_ids!(manager, tags, tag_ids)`.
///
/// An empty `IdFilter` matches the whole table, so no ids means no query.
macro_rules! by_ids {
    ($manager:expr, $repo:ident, $ids:expr) => {{
        let ids: std::collections::HashSet<_> = $ids.into_iter().collect();
        if ids.is_empty() {
            Vec::new()
        } else {
            use post_archiver::query::Query as _;
            let mut query = $manager.$repo().query();
            query.ids.extend(ids);
            query.query()?
        }
    }};
}
pub(crate) use by_ids;

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
        self.platforms = by_ids!(manager, platforms, ids);
        Ok(self)
    }

    pub fn file_metas(
        mut self,
        manager: &PostArchiverManager,
        ids: impl IntoIterator<Item = FileMetaId>,
    ) -> ApiResult<Self> {
        self.file_metas = by_ids!(manager, file_metas, ids);
        Ok(self)
    }
}
