use axum::{
    Json,
    extract::{Path, State},
};
use chrono::{DateTime, Utc};
use post_archiver::{
    Author, AuthorId, Collection, CollectionId, Comment, Content, FileMeta, FileMetaId, PlatformId,
    Post, PostId, Tag, TagId, impl_from_query, query::PostArchiverQueryExt,
};
use serde::Serialize;
use ts_rs::TS;

use super::{
    AppState,
    error::{ApiError, ApiResult},
    relation::{WithRelations, by_ids},
};

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

/// A post with its relation columns, read in one query.
struct PostDetail {
    id: PostId,
    title: String,
    content: Vec<Content>,
    source: Option<String>,
    updated: DateTime<Utc>,
    published: DateTime<Utc>,
    thumb: Option<FileMetaId>,
    platform: Option<PlatformId>,
    comments: Vec<Comment>,
    tags: Vec<TagId>,
    authors: Vec<AuthorId>,
    collections: Vec<CollectionId>,
    file_metas: Vec<FileMeta>,
}

impl_from_query! {
    PostDetail extends Post {
        id: id,
        title: title,
        content: content,
        source: source,
        updated: updated,
        published: published,
        thumb: thumb,
        platform: platform,
        comments: comments,
        tags: post_tags,
        authors: author_posts,
        collections: collection_posts,
        file_metas: file_metas,
    }
}

pub async fn get_post_handler(
    Path(id): Path<PostId>,
    State(state): State<AppState>,
) -> ApiResult<Json<WithRelations<PostResponse>>> {
    let manager = state.manager();

    let post: PostDetail = manager.posts().get(id)?.ok_or(ApiError::NotFound)?;
    let tags: Vec<Tag> = by_ids!(manager, tags, post.tags);
    let authors: Vec<Author> = by_ids!(manager, authors, post.authors);
    let collections: Vec<Collection> = by_ids!(manager, collections, post.collections);

    let platforms: Vec<PlatformId> = post
        .platform
        .into_iter()
        .chain(tags.iter().filter_map(|t| t.platform))
        .collect();

    let response = PostResponse {
        id: post.id,
        title: post.title,
        content: post.content,
        source: post.source,
        updated: post.updated,
        published: post.published,
        thumb: post.thumb,
        platform: post.platform,
        comments: post.comments,
        tags,
        authors,
        collections,
    };

    let mut response = WithRelations::new(response).platforms(&manager, platforms)?;
    // Every file the post shows (content and thumb) belongs to the post itself.
    response.file_metas = post.file_metas;
    Ok(Json(response))
}
