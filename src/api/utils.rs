use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub limit: Option<u64>,
    pub page: Option<u64>,
}

impl Pagination {
    pub fn limit(&self) -> u64 {
        self.limit.unwrap_or(20)
    }
    pub fn page(&self) -> u64 {
        self.page.unwrap_or(0)
    }
    pub fn params(&self) -> [(&'static str, u64); 2] {
        let limit = self.limit();
        let page = self.page() * limit;
        [(":limit", limit), (":offset", page)]
    }
}

pub mod post_preview {
    use chrono::{DateTime, Utc};
    use post_archiver::{FileMetaId, Post, PostId, impl_from_query};
    use serde::Serialize;
    use ts_rs::TS;

    use crate::api::relation::RequireRelations;

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
            id: "id",
            title: "title",
            thumb: "thumb",
            updated: "updated",
        }
    }

    impl RequireRelations for PostPreview {
        fn file_metas(&self) -> Vec<FileMetaId> {
            self.thumb.into_iter().collect()
        }
    }
}

pub mod list {
    

    
    use post_archiver::{AuthorId, CollectionId, FileMetaId, PlatformId, TagId, error::Result, manager::PostArchiverConnection, query::{BaseFilter, FromQuery, Query, Queryer, RawSql, Sortable}};
    use serde::Serialize;
    use ts_rs::TS;

    use crate::api::relation::RequireRelations;

    #[derive(Debug)]
    pub struct WithCachedTotal<Q> {
        inner: Q,
        total: Option<usize>,
    }

    impl<Q> WithCachedTotal<Q> {
        pub fn new(inner: Q, cached: Option<usize>) -> Self {
            Self { inner, total: cached }
        }
    }

    impl<Q: Query + BaseFilter> Query for WithCachedTotal<Q> {
        type Wrapper<T> = ListResponse<Q::Wrapper<T>>;
        type Based = <Q as Query>::Based;

        fn query_with_context<T: FromQuery<Based = Self::Based>>(
            self,
            sql: RawSql<T>,
        ) -> Result<Self::Wrapper<T>> {
            let total = match self.total {
                Some(total) => total,
                None => self.inner.count()? as usize
            };
            let list = self.inner.query_with_context(sql)?;
            Ok(ListResponse { list, total })
        }
    }

    impl<T: BaseFilter> BaseFilter for WithCachedTotal<T> {
        type Based = T::Based;

        fn update_sql<U: FromQuery<Based = Self::Based>>(&self, sql: RawSql<U>) -> RawSql<U> {
            self.inner.update_sql(sql)
        }

        fn queryer(&self) -> &Queryer<'_, impl PostArchiverConnection> {
            self.inner.queryer()
        }
    }

    impl<T: Sortable> Sortable for WithCachedTotal<T> {
        type SortField = T::SortField;
    }

    #[derive(Debug, Clone, Serialize, TS)]
    #[ts(export)]
    pub struct ListResponse<T> {
        pub list: T,
        pub total: usize,
    }

    impl<T: RequireRelations> RequireRelations for ListResponse<T> {
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
}
