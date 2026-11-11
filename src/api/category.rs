//! `/tags`, `/authors`, `/collections`, `/platforms`: a searchable page of each, and one by id.

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use axum_extra::extract::Query;
use chrono::{DateTime, Utc};
use post_archiver::{
    Alias, Author, AuthorId, Collection, CollectionId, FileMetaId, Platform, PlatformId, Tag,
    TagId, impl_from_query,
    query::{
        PostArchiverQueryExt, author::AuthorSort, collection::CollectionSort,
        platform::PlatformSort, tag::TagSort,
    },
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    AppState,
    error::{ApiError, ApiResult},
    relation::WithRelations,
    utils::{Pagination, Totalled},
};

pub fn wrap_category_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/tags", get(list_tags))
        .route("/tags/{id}", get(get_tag))
        .route("/authors", get(list_authors))
        .route("/authors/{id}", get(get_author))
        .route("/collections", get(list_collections))
        .route("/collections/{id}", get(get_collection))
        .route("/platforms", get(list_platforms))
        .route("/platforms/{id}", get(get_platform))
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CategoryOrderBy {
    Id,
    Name,
    /// Only authors have `updated`; the others fall back to `Id`.
    Updated,
    Random,
}

#[derive(Debug, Deserialize)]
pub struct Filter {
    #[serde(default)]
    pub search: String,
    pub order_by: Option<CategoryOrderBy>,
}

type ListResponse<T> = ApiResult<Json<WithRelations<Totalled<Vec<T>>>>>;
type GetResponse<T> = ApiResult<Json<WithRelations<T>>>;

// ── tags ─────────────────────────────────────────────────────────────────────

async fn list_tags(
    Query(filter): Query<Filter>,
    Query(pagination): Query<Pagination>,
    State(state): State<AppState>,
) -> ListResponse<Tag> {
    let manager = state.manager();
    let mut query = manager.tags().query();
    query.name.contains(&filter.search);
    match filter.order_by.unwrap_or(CategoryOrderBy::Name) {
        CategoryOrderBy::Id | CategoryOrderBy::Updated => query.order.desc(TagSort::Id),
        CategoryOrderBy::Name => query.order.asc(TagSort::Name),
        CategoryOrderBy::Random => query.order.random(),
    };
    query.pagination = pagination.to_query();

    let page = Totalled::<Vec<Tag>>::query(&query)?;
    let platforms: Vec<PlatformId> = page.items.iter().filter_map(|t| t.platform).collect();
    Ok(Json(
        WithRelations::new(page).platforms(&manager, platforms)?,
    ))
}

async fn get_tag(Path(id): Path<TagId>, State(state): State<AppState>) -> GetResponse<Tag> {
    let manager = state.manager();
    let tag: Tag = manager.tags().get(id)?.ok_or(ApiError::NotFound)?;
    let platform = tag.platform;
    Ok(Json(WithRelations::new(tag).platforms(&manager, platform)?))
}

// ── authors ──────────────────────────────────────────────────────────────────

/// An author together with its aliases (one query, through the `author_aliases` relation column).
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct AuthorResponse {
    pub id: AuthorId,
    pub name: String,
    pub thumb: Option<FileMetaId>,
    pub updated: DateTime<Utc>,
    pub aliases: Vec<Alias>,
}

impl_from_query! {
    AuthorResponse extends Author {
        id: id,
        name: name,
        thumb: thumb,
        updated: updated,
        aliases: author_aliases,
    }
}

async fn list_authors(
    Query(filter): Query<Filter>,
    Query(pagination): Query<Pagination>,
    State(state): State<AppState>,
) -> ListResponse<Author> {
    let manager = state.manager();
    let mut query = manager.authors().query();
    query.name.contains(&filter.search);
    match filter.order_by.unwrap_or(CategoryOrderBy::Updated) {
        CategoryOrderBy::Id => query.order.desc(AuthorSort::Id),
        CategoryOrderBy::Name => query.order.asc(AuthorSort::Name),
        CategoryOrderBy::Updated => query.order.desc(AuthorSort::Updated),
        CategoryOrderBy::Random => query.order.random(),
    };
    query.pagination = pagination.to_query();

    let page = Totalled::<Vec<Author>>::query(&query)?;
    let thumbs: Vec<FileMetaId> = page.items.iter().filter_map(|a| a.thumb).collect();
    Ok(Json(WithRelations::new(page).file_metas(&manager, thumbs)?))
}

async fn get_author(
    Path(id): Path<AuthorId>,
    State(state): State<AppState>,
) -> GetResponse<AuthorResponse> {
    let manager = state.manager();
    let author: AuthorResponse = manager.authors().get(id)?.ok_or(ApiError::NotFound)?;
    let platforms: Vec<PlatformId> = author.aliases.iter().map(|a| a.platform).collect();
    let thumb = author.thumb;
    Ok(Json(
        WithRelations::new(author)
            .platforms(&manager, platforms)?
            .file_metas(&manager, thumb)?,
    ))
}

// ── collections ──────────────────────────────────────────────────────────────

async fn list_collections(
    Query(filter): Query<Filter>,
    Query(pagination): Query<Pagination>,
    State(state): State<AppState>,
) -> ListResponse<Collection> {
    let manager = state.manager();
    let mut query = manager.collections().query();
    query.name.contains(&filter.search);
    match filter.order_by.unwrap_or(CategoryOrderBy::Name) {
        CategoryOrderBy::Id | CategoryOrderBy::Updated => query.order.desc(CollectionSort::Id),
        CategoryOrderBy::Name => query.order.asc(CollectionSort::Name),
        CategoryOrderBy::Random => query.order.random(),
    };
    query.pagination = pagination.to_query();

    let page = Totalled::<Vec<Collection>>::query(&query)?;
    let thumbs: Vec<FileMetaId> = page.items.iter().filter_map(|c| c.thumb).collect();
    Ok(Json(WithRelations::new(page).file_metas(&manager, thumbs)?))
}

async fn get_collection(
    Path(id): Path<CollectionId>,
    State(state): State<AppState>,
) -> GetResponse<Collection> {
    let manager = state.manager();
    let collection: Collection = manager.collections().get(id)?.ok_or(ApiError::NotFound)?;
    let thumb = collection.thumb;
    Ok(Json(
        WithRelations::new(collection).file_metas(&manager, thumb)?,
    ))
}

// ── platforms ────────────────────────────────────────────────────────────────

async fn list_platforms(
    Query(filter): Query<Filter>,
    Query(pagination): Query<Pagination>,
    State(state): State<AppState>,
) -> ListResponse<Platform> {
    let manager = state.manager();
    let mut query = manager.platforms().query();
    query.name.contains(&filter.search);
    match filter.order_by.unwrap_or(CategoryOrderBy::Name) {
        CategoryOrderBy::Id | CategoryOrderBy::Updated => query.order.desc(PlatformSort::Id),
        CategoryOrderBy::Name => query.order.asc(PlatformSort::Name),
        CategoryOrderBy::Random => query.order.random(),
    };
    query.pagination = pagination.to_query();

    Ok(Json(WithRelations::new(Totalled::query(&query)?)))
}

async fn get_platform(
    Path(id): Path<PlatformId>,
    State(state): State<AppState>,
) -> GetResponse<Platform> {
    let manager = state.manager();
    let platform: Platform = manager.platforms().get(id)?.ok_or(ApiError::NotFound)?;
    Ok(Json(WithRelations::new(platform)))
}
