use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use axum_extra::extract::Query;
use post_archiver::{
    AuthorId, CollectionId, PlatformId, TagId,
    query::{Pagination as QueryPagination, PostArchiverQueryExt, Query as _, post::PostSort},
};
use serde::{Deserialize, Serialize};

use super::{
    AppState,
    post::get_post_handler,
    relation::WithRelations,
    utils::{Pagination, Totalled, post_preview::PostPreview},
};

pub fn wrap_posts_route(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/posts", get(list_posts_handler))
        .route("/posts/{id}", get(get_post_handler))
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "lowercase")]
pub enum PostOrderBy {
    Id,
    #[default]
    Updated,
    Random,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Default)]
pub struct SearchQuery {
    #[serde(default)]
    search: String,
    #[serde(default)]
    tags: Vec<TagId>,
    #[serde(default)]
    collections: Vec<CollectionId>,
    #[serde(default)]
    authors: Vec<AuthorId>,
    #[serde(default)]
    platforms: Vec<PlatformId>,
    #[serde(default)]
    order_by: PostOrderBy,
}

pub async fn list_posts_handler(
    Query(pagination): Query<Pagination>,
    Query(searchs): Query<SearchQuery>,
    State(state): State<AppState>,
) -> Result<Json<WithRelations<Totalled<Vec<PostPreview>>>>, StatusCode> {
    let manager = state.manager();

    let mut query = manager.posts().query();

    query.title.contains(&searchs.search);
    query.authors.extend(searchs.authors.clone());
    query.tags.extend(searchs.tags.clone());
    query.collections.extend(searchs.collections.clone());
    query.platforms.extend(searchs.platforms.clone());

    match searchs.order_by {
        PostOrderBy::Id => query.order.desc(PostSort::Id),
        PostOrderBy::Updated => query.order.desc(PostSort::Updated),
        PostOrderBy::Random => query.order.random(),
    };
    query.pagination = QueryPagination::default()
        .limit(pagination.limit())
        .page(pagination.page());

    let total = query
        .count()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let items = query
        .query::<PostPreview>()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let result = Totalled { items, total };

    // Cache the total if it was not cached before
    WithRelations::new(&manager, result)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .map(Json::from)
}
