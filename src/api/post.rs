use std::collections::HashSet;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use post_archiver::{
    content, Author, Collection, Comment, Content, FileMeta, FileMetaId, Platform, Post, PostId,
    Tag,
};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{utils::{get_file_metas, WithThumb}, AppState};

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct PostJson {
    pub id: PostId,
    pub title: String,
    pub content: Vec<ContentJson>,
    pub source: Option<String>,
    pub updated: DateTime<Utc>,
    pub published: DateTime<Utc>,
    pub thumb: Option<FileMeta>,
    pub platform: Option<Platform>,

    pub tags: Vec<Tag>,
    pub authors: Vec<WithThumb<Author>>,
    pub collections: Vec<WithThumb<Collection>>,
    pub comments: Vec<Comment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(untagged)]
#[ts(export)]
pub enum ContentJson {
    Text(String),
    File(FileMeta),
}

pub async fn get_post_api(
    Path(id): Path<PostId>,
    State(state): State<AppState>,
) -> Result<Json<PostJson>, StatusCode> {
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

    let platform = post
        .platform
        .map(|p| {
            manager
                .get_platform(&p)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        })
        .transpose()?;

    let file_ids: HashSet<FileMetaId> = post
        .content
        .iter()
        .cloned()
        .filter_map(|content| match content {
            Content::File(file_meta) => Some(file_meta),
            _ => None,
        })
        .chain(post.thumb.iter().cloned())
        .chain(authors.iter().filter_map(|a| a.thumb))
        .chain(collections.iter().filter_map(|c| c.thumb))
        .collect();

    let file_metas =
        get_file_metas(&manager, file_ids).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let thumb = post.thumb.and_then(|thumb| file_metas.get(&thumb).cloned());

    let authors = authors
        .into_iter()
        .map(|author| {
            let thumb = author
                .thumb
                .and_then(|thumb| file_metas.get(&thumb).cloned());
            WithThumb {
                category: author,
                thumb,
            }
        })
        .collect::<Vec<_>>();

    let collections = collections
        .into_iter()
        .map(|collection| {
            let thumb = collection
                .thumb
                .and_then(|thumb| file_metas.get(&thumb).cloned());
            WithThumb {
                category: collection,
                thumb,
            }
        })
        .collect::<Vec<_>>();

    let content = post
        .content
        .into_iter()
        .filter_map(|c| match c {
            content::Content::Text(text) => Some(ContentJson::Text(text)),
            content::Content::File(id) => file_metas.get(&id).cloned().map(ContentJson::File),
        })
        .collect::<Vec<_>>();

    Ok(Json(PostJson {
        id: post.id,
        title: post.title,
        content,
        thumb,
        platform,
        source: post.source,
        updated: post.updated,
        published: post.published,
        comments: post.comments,

        tags,
        authors,
        collections,
    }))
}
