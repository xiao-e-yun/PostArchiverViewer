use mini_moka::sync::Cache;
use post_archiver::{Author, AuthorId};
use rusqlite::Row;

use super::{Category, CategoryApiRouter, CategoryPostsApiRouter};

impl Category for Author {
    type Id = AuthorId;
    const TABLE_NAME: &'static str = "authors";
    const ORDER_BY: &'static str = "ORDER BY updated DESC";
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Author::from_row(row)
    }

    fn thumb(&self) -> Option<post_archiver::FileMetaId> {
        self.thumb
    }
}

impl CategoryApiRouter for Author {
    const ROUTE_NAME: &'static str = "authors";
}

impl CategoryPostsApiRouter for Author {
    const JOIN_RELATION: &'static str = "JOIN author_posts ON author_posts.post = posts.id";
    const FILTER: &'static str = "author_posts.author";

    fn post_cache(
            state: &crate::api::AppState,
    ) -> &Cache<Self::Id, usize> {
        &state.caches.authors
    }
}
