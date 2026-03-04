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

mod totalled {
    use post_archiver::{*, query::Totalled};

    use crate::api::relation::RequireRelations;

    impl<T: RequireRelations> RequireRelations for Totalled<T> {
        fn authors(&self) -> Vec<AuthorId> {
            self.items.authors()
        }
        fn collections(&self) -> Vec<CollectionId> {
            self.items.collections()
        }
        fn platforms(&self) -> Vec<PlatformId> {
            self.items.platforms()
        }
        fn tags(&self) -> Vec<TagId> {
            self.items.tags()
        }
        fn file_metas(&self) -> Vec<FileMetaId> {
            self.items.file_metas()
        }
    }
}
