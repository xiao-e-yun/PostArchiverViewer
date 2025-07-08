use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use mini_moka::sync::Cache;
use post_archiver::{Alias, Author, AuthorId, PlatformId};
use rusqlite::Row;

use crate::api::{
    category::{get_category_handler, list_category_posts_handler},
    relation::{RequireRelations, WithRelations},
    utils::ListResponse,
    AppState,
};

use super::{Category, CategoryApiRouter, CategoryPostsApiRouter};

impl RequireRelations for Author {
    fn file_metas(&self) -> Vec<post_archiver::FileMetaId> {
        self.thumb.into_iter().collect()
    }
}

impl Category for Author {
    type Id = AuthorId;
    const TABLE_NAME: &'static str = "authors";
    const ORDER_BY: &'static str = "ORDER BY updated DESC";

    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Author::from_row(row)
    }
}

impl CategoryApiRouter for Author {
    const ROUTE_NAME: &'static str = "authors";
}

impl CategoryPostsApiRouter for Author {
    const JOIN_RELATION: &'static str = "JOIN author_posts ON author_posts.post = posts.id";
    const FILTER: &'static str = "author_posts.author";

    fn post_cache(state: &crate::api::AppState) -> &Cache<Self::Id, usize> {
        &state.caches.authors
    }

    fn wrap_category_and_posts_route(router: Router<AppState>) -> Router<AppState> {
        Self::wrap_category_route(router)
            .route(
                &format!("/{}/{{id}}", Self::ROUTE_NAME),
                get(get_category_handler::<Self>),
            )
            .route(
                &format!("/{}/{{id}}/posts", Self::ROUTE_NAME),
                get(list_category_posts_handler::<Self>),
            )
            .route(
                &format!("/{}/{{id}}/aliases", Self::ROUTE_NAME),
                get(author_aliases_handler),
            )
    }
}

pub async fn author_aliases_handler(
    State(state): State<AppState>,
    Path(id): Path<AuthorId>,
) -> Result<Json<WithRelations<ListResponse<Alias>>>, StatusCode> {
    let manager = &state.manager();
    let list = manager
        .list_author_aliases(id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let total = list.len();

    WithRelations::new(manager, ListResponse { list, total })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .map(Json::from)
}

impl RequireRelations for Alias {
    fn platforms(&self) -> Vec<PlatformId> {
        vec![self.platform]
    }
}
