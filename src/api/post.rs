use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use axum_extra::extract::Query;
use chrono::{DateTime, Utc};
use post_archiver::{
    Author, Collection, Comment, Content, FileMeta, FileMetaId, PlatformId, Post, PostId, Tag,
};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::api::{
    search::{get_search_api, SearchQuery},
    utils::{Pagination, PostListResponse},
    AppState,
};

use super::relation::{RequireRelations, WithRelations};

pub fn wrap_posts_route(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/posts", get(list_posts_handler))
        .route("/posts/{id}", get(get_post_handler))
}

async fn list_posts_handler(
    Query(pagination): Query<Pagination>,
    State(state): State<AppState>,
) -> Result<Json<WithRelations<PostListResponse>>, StatusCode> {
    get_search_api(
        Query(pagination),
        Query(SearchQuery::default()),
        State(state),
    )
    .await
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct PostResponse {
    pub id: PostId,
    pub title: String,
    pub content: Vec<Content>,
    pub source: Option<String>,
    pub updated: DateTime<Utc>,
    pub published: DateTime<Utc>,
    pub thumb: Option<FileMetaId>,
    pub platform: Option<PlatformId>,

    pub tags: Vec<Tag>,
    pub authors: Vec<Author>,
    pub collections: Vec<Collection>,
    pub comments: Vec<Comment>,
}

impl RequireRelations for PostResponse {
    fn platforms(&self) -> Vec<PlatformId> {
        self.platform
            .iter()
            .cloned()
            .chain(self.tags.iter().filter_map(|a| a.platform))
            .collect()
    }
    fn file_metas(&self) -> Vec<FileMetaId> {
        self.content
            .iter()
            .filter_map(|content| match content {
                Content::File(file_meta) => Some(*file_meta),
                _ => None,
            })
            .chain(self.thumb.iter().cloned())
            .chain(self.authors.iter().flat_map(|a| a.thumb))
            .chain(self.collections.iter().flat_map(|c| c.thumb))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(untagged)]
#[ts(export)]
pub enum ContentJson {
    Text(String),
    File(FileMeta),
}

pub async fn get_post_handler(
    Path(id): Path<PostId>,
    State(state): State<AppState>,
) -> Result<Json<WithRelations<PostResponse>>, StatusCode> {
    let manager = state.manager();

    let mut stmt = manager
        .conn()
        .prepare_cached("SELECT * FROM posts WHERE id = ?")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some(post) = stmt
        .query_row([id], Post::from_row)
        .optional()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::NOT_FOUND);
    };

    let tags = manager
        .list_post_tags(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let authors = manager
        .list_post_authors(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let collections = manager
        .list_post_collections(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    WithRelations::new(
        &manager,
        PostResponse {
            id: post.id,
            title: post.title,
            content: post.content,
            thumb: post.thumb,
            platform: post.platform,
            source: post.source,
            updated: post.updated,
            published: post.published,
            comments: post.comments,
            tags,
            authors,
            collections,
        },
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    .map(Json::from)
}
