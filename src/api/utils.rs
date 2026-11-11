use post_archiver::query::{FromQuery, Pagination as QueryPagination, Query};
use serde::{Deserialize, Serialize};

use super::error::ApiResult;

/// `?limit=&page=` of list endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub limit: Option<u64>,
    pub page: Option<u64>,
}

impl Pagination {
    pub fn to_query(&self) -> QueryPagination {
        QueryPagination::default()
            .limit(self.limit.unwrap_or(20))
            .page(self.page.unwrap_or(0))
    }
}

/// A page of items together with the total number of matches (ignoring pagination).
#[derive(Debug, Clone, Serialize)]
pub struct Totalled<T> {
    pub items: T,
    pub total: u64,
}

impl<T> Totalled<Vec<T>> {
    pub fn query<Q: Query>(query: &Q) -> ApiResult<Self>
    where
        T: FromQuery<Based = Q::Source>,
    {
        Ok(Self {
            total: query.count()?,
            items: query.query()?,
        })
    }
}

pub mod post_preview {
    use chrono::{DateTime, Utc};
    use post_archiver::{FileMetaId, Post, PostId, impl_from_query};
    use serde::Serialize;
    use ts_rs::TS;

    #[derive(Debug, Clone, Serialize, TS)]
    #[ts(export)]
    pub struct PostPreview {
        pub id: PostId,
        pub title: String,
        pub thumb: Option<FileMetaId>,
        pub updated: DateTime<Utc>,
    }

    impl_from_query! {
        PostPreview extends Post {
            id: id,
            title: title,
            thumb: thumb,
            updated: updated,
        }
    }
}
