use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use post_archiver::{
    Author, Collection, Comment, Content, FileMetaId, PlatformId, PostId, Tag, query::Query,
};
use serde::Serialize;
use ts_rs::TS;

use crate::api::AppState;

use super::relation::{RequireRelations, WithRelations};

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

pub async fn get_post_handler(
    Path(id): Path<PostId>,
    State(state): State<AppState>,
) -> Result<Json<WithRelations<PostResponse>>, StatusCode> {
    let manager = state.manager();

    let Some(post) = manager
        .get_post(id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::NOT_FOUND);
    };

    let binded_post = manager.bind(id);

    macro_rules! query_relation {
        ($list_method:ident, $query_method:ident) => {{
            let ids = binded_post
                .$list_method()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            if ids.is_empty() {
                Vec::new()
            } else {
                let mut query = manager.$query_method();
                query.ids.extend(ids);
                query
                    .query()
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            }
        }};
    }

    let tags = query_relation!(list_tags, tags);
    let authors = query_relation!(list_authors, authors);
    let collections = query_relation!(list_collections, collections);

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
