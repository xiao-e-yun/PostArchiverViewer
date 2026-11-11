use axum::{Json, Router, extract::State, routing::get};
use axum_extra::extract::Query;
use post_archiver::{
    AuthorId, CollectionId, FileMetaId, PlatformId, TagId,
    query::{PostArchiverQueryExt, post::PostSort},
};
use serde::{Deserialize, Serialize};

use super::{
    AppState,
    error::ApiResult,
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
) -> ApiResult<Json<WithRelations<Totalled<Vec<PostPreview>>>>> {
    let manager = state.manager();

    let mut query = manager.posts().query();
    query.title.contains(&searchs.search);
    query.authors.extend(searchs.authors);
    query.tags.extend(searchs.tags);
    query.collections.extend(searchs.collections);
    query.platforms.extend(searchs.platforms);
    match searchs.order_by {
        PostOrderBy::Id => query.order.desc(PostSort::Id),
        PostOrderBy::Updated => query.order.desc(PostSort::Updated),
        PostOrderBy::Random => query.order.random(),
    };
    query.pagination = pagination.to_query();

    let page = Totalled::<Vec<PostPreview>>::query(&query)?;
    let thumbs: Vec<FileMetaId> = page.items.iter().filter_map(|p| p.thumb).collect();
    Ok(Json(WithRelations::new(page).file_metas(&manager, thumbs)?))
}
