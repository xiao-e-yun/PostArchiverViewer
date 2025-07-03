use mini_moka::sync::Cache;
use post_archiver::{PlatformId, Tag, TagId};
use rusqlite::Row;

use crate::api::relation::RequireRelations;

use super::{Category, CategoryApiRouter, CategoryPostsApiRouter};

impl RequireRelations for Tag {
    fn platforms(&self) -> Vec<PlatformId> {
        self.platform.into_iter().collect()
    }
}

impl Category for Tag {
    type Id = TagId;
    const TABLE_NAME: &'static str = "tags";
    const ORDER_BY: &'static str = "ORDER BY id DESC";

    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Tag::from_row(row)
    }
}

impl CategoryApiRouter for Tag {
    const ROUTE_NAME: &'static str = "tags";
}

impl CategoryPostsApiRouter for Tag {
    const JOIN_RELATION: &'static str = "JOIN post_tags ON post_tags.post = posts.id";
    const FILTER: &'static str = "post_tags.tag";

    fn post_cache(
            state: &crate::api::AppState,
    ) -> &Cache<Self::Id, usize> {
        &state.caches.tags
    }
}
