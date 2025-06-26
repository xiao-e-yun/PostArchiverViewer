use mini_moka::sync::Cache;
use post_archiver::{Collection, CollectionId};
use rusqlite::Row;

use super::{Category, CategoryApiRouter, CategoryPostsApiRouter};

impl Category for Collection {
    type Id = CollectionId;
    const TABLE_NAME: &'static str = "collections";
    const ORDER_BY: &'static str = "ORDER BY id DESC";
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Collection::from_row(row)
    }

    fn thumb(&self) -> Option<post_archiver::FileMetaId> {
        self.thumb
    }
}

impl CategoryApiRouter for Collection {
    const ROUTE_NAME: &'static str = "collections";
}

impl CategoryPostsApiRouter for Collection {
    const JOIN_RELATION: &'static str = "JOIN collection_posts ON collection_posts.post = posts.id";
    const FILTER: &'static str = "collection_posts.collection";

    fn post_cache(
        state: &crate::api::AppState,
    ) -> &Cache<Self::Id, usize> {
        &state.caches.collections
    }
}
