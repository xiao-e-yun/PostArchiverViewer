use mini_moka::sync::Cache;
use post_archiver::{Platform, PlatformId};
use rusqlite::Row;

use crate::api::relation::RequireRelations;

use super::{Category, CategoryApiRouter, CategoryPostsApiRouter};

impl RequireRelations for Platform {}

impl Category for Platform {
    type Id = PlatformId;
    const TABLE_NAME: &'static str = "platforms";
    const ORDER_BY: &'static str = "ORDER BY id DESC";

    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Platform::from_row(row)
    }
}

impl CategoryApiRouter for Platform {
    const ROUTE_NAME: &'static str = "platforms";
}

impl CategoryPostsApiRouter for Platform {
    const FILTER: &'static str = "posts.platform";
    const JOIN_RELATION: &'static str = "";

    fn post_cache(
            state: &crate::api::AppState,
    ) -> &Cache<Self::Id, usize> {
        &state.caches.platforms
    }
}
