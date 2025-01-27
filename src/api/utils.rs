use std::ops::Deref;

use chrono::{DateTime, Utc};
use post_archiver::{Author, AuthorId, Comment, Content, FileMetaId, Link, Post, PostId, PostTagId, Tag};
use rusqlite::{OptionalExtension, Row, Transaction};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::error;
use ts_rs::TS;

use super::AppState;

pub trait FromRow {
    fn from_row(state: &AppState, row: &Row) -> Result<Self, rusqlite::Error>
    where
        Self: Sized;
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AuthorJson {
    pub id: AuthorId,
    pub name: String,
    pub links: Vec<Link>,
    pub thumb: Option<FileMetaJson>,
    pub updated: DateTime<Utc>,
}

impl AuthorJson {
    pub fn from_id(
        state: &AppState,
        tx: &Transaction,
        id: AuthorId,
    ) -> Result<Self, rusqlite::Error> {
        let mut stmt = tx.prepare_cached("SELECT * FROM authors WHERE id = ?")?;

        match stmt
            .query_row([id], |row| Author::from_row(state, row))
            .optional()?
        {
            Some(author) => Self::resolve(state, tx, author),
            None => {
                error!("Author {} not found", id);
                Err(rusqlite::Error::UnwindingPanic)
            }
        }
    }

    pub fn resolve(
        state: &AppState,
        tx: &Transaction,
        author: Author,
    ) -> Result<Self, rusqlite::Error> {
        let thumb = author
            .thumb
            .map(|id| FileMetaJson::resolve(state, tx, id))
            .transpose()?;

        Ok(AuthorJson {
            id: author.id,
            name: author.name,
            links: author.links,
            thumb,
            updated: author.updated,
        })
    }
}

impl FromRow for Author {
    fn from_row(_state: &AppState, row: &Row) -> Result<Self, rusqlite::Error> {
        let id: AuthorId = row.get(0)?;
        let name: String = row.get(1)?;
        let thumb: Option<FileMetaId> = row.get(3)?;
        let updated: DateTime<Utc> = row.get(4)?;

        let links: String = row.get(2)?;
        let Ok(links) = serde_json::from_str(&links) else {
            error!("Failed to parse links for author {}", id);
            return Err(rusqlite::Error::UnwindingPanic);
        };

        Ok(Author {
            id,
            name,
            thumb,
            updated,
            links,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize,TS)]
#[ts(export)]
pub struct PostJson {
    pub id: PostId,
    pub author: AuthorJson,
    pub source: Option<String>,
    pub title: String,
    pub content: Vec<ContentJson>,
    pub thumb: Option<FileMetaJson>,
    pub comments: Vec<Comment>,
    pub updated: DateTime<Utc>,
    pub published: DateTime<Utc>,
    pub tags: Vec<TagJson>,
}

#[derive(Debug, Clone, Serialize, Deserialize,TS)]
#[serde(untagged)]
#[ts(export)]
pub enum ContentJson {
    Text(String),
    File(FileMetaJson),
}

impl PostJson {
    pub fn from_id(
        state: &AppState,
        tx: &Transaction,
        id: PostId,
    ) -> Result<Self, rusqlite::Error> {
        let mut stmt = tx.prepare_cached("SELECT * FROM posts WHERE id = ?")?;

        match stmt
            .query_row([id], |row| Post::from_row(state, row))
            .optional()?
        {
            Some(post) => Self::resolve(state, tx, post),
            None => {
                error!("Author {} not found", id);
                Err(rusqlite::Error::UnwindingPanic)
            }
        }
    }

    pub fn resolve(
        state: &AppState,
        tx: &Transaction,
        post: Post,
    ) -> Result<Self, rusqlite::Error> {

        let author = AuthorJson::from_id(state, tx, post.author)?;
        
        let content = post.content.into_iter().map(|content|{
            Ok(match content {
                Content::Text(text) => ContentJson::Text(text),
                Content::File(id) => ContentJson::File(FileMetaJson::resolve(state, tx, id)?),
            })
        }).collect::<Result<Vec<ContentJson>,rusqlite::Error>>()?;

        let thumb = post
            .thumb
            .map(|thumb| FileMetaJson::resolve(state, tx, thumb))
            .transpose()?;

        let tags = TagJson::from_post(state, tx, post.id)?;

        Ok(PostJson {
            id: post.id,
            author,
            source: post.source,
            title: post.title,
            content,
            thumb,
            comments: post.comments,
            updated: post.updated,
            published: post.published,
            tags,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize,TS)]
#[ts(export)]
pub struct PostMiniJson {
    pub id: PostId,
    pub author: AuthorJson,
    pub title: String,
    pub thumb: Option<FileMetaJson>,
    pub updated: DateTime<Utc>,
}

impl PostMiniJson {
    pub fn resolve(
        state: &AppState,
        tx: &Transaction,
        post: Post,
    ) -> Result<Self, rusqlite::Error> {
        let author = AuthorJson::from_id(state, tx, post.author)?;

        let thumb = post
            .thumb
            .map(|thumb| FileMetaJson::resolve(state, tx, thumb))
            .transpose()?;

        Ok(PostMiniJson {
            id: post.id,
            author,
            title: post.title,
            thumb,
            updated: post.updated,
        })
    }
}

impl FromRow for Post {
    fn from_row(_state: &AppState, row: &Row) -> Result<Self, rusqlite::Error>
    where
        Self: Sized,
    {
        let id: PostId = row.get(0)?;
        let author: AuthorId = row.get(1)?;
        let source: Option<String> = row.get(2)?;
        let title: String = row.get(3)?;

        let content: String = row.get(4)?;
        let Ok(content) = serde_json::from_str(&content) else {
            error!("Failed to parse content for post {}", id);
            return Err(rusqlite::Error::UnwindingPanic);
        };

        let thumb = row.get(5)?;

        let comments: String = row.get(6)?;
        let Ok(comments) = serde_json::from_str(&comments) else {
            error!("Failed to parse comments for post {}", id);
            return Err(rusqlite::Error::UnwindingPanic);
        };

        let updated: DateTime<Utc> = row.get(7)?;
        let published: DateTime<Utc> = row.get(8)?;

        Ok(Post {
            id,
            author,
            source,
            title,
            content,
            thumb,
            comments,
            updated,
            published,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize,TS)]
#[ts(export)]
pub struct FileMetaJson {
    pub id: FileMetaId,
    pub url: String,
    pub mime: String,
    #[ts(type = "Record<string,any>")]
    pub extra: Value,
}

impl FileMetaJson {
    pub fn resolve(
        state: &AppState,
        tx: &Transaction,
        id: FileMetaId,
    ) -> Result<Self, rusqlite::Error> {
        let mut stmt = tx.prepare_cached("SELECT * FROM file_metas WHERE id = ?")?;

        match stmt
            .query_row([id], |row| FileMetaJson::from_row(state, row))
            .optional()?
        {
            Some(data) => Ok(data),
            // return
            None => {
                let id = FileMetaId(0);
                let url = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='1em' height='1em' viewBox='0 0 24 24'%3E%3Cpath fill='none' stroke='currentColor' stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M3 7v4a1 1 0 0 0 1 1h3m0-5v10m3-9v8a1 1 0 0 0 1 1h2a1 1 0 0 0 1-1V8a1 1 0 0 0-1-1h-2a1 1 0 0 0-1 1m7-1v4a1 1 0 0 0 1 1h3m0-5v10'/%3E%3C/svg%3E".to_string();
                let mime = "image/svg+xml".to_string();
                let extra = json!({});
                Ok(FileMetaJson {
                    id,
                    url,
                    mime,
                    extra,
                })
            }
        }
    }
}

impl FromRow for FileMetaJson {
    fn from_row(state: &AppState, row: &Row) -> Result<Self, rusqlite::Error> {
        let id: FileMetaId = row.get(0)?;
        let filename: String = row.get(1)?;
        let author: AuthorId = row.get(2)?;
        let post: PostId = row.get(3)?;
        let mime: String = row.get(4)?;

        let extra = row.get(5)?;
        let extra = parse_json(extra)?;

        let url = format!(
            "{}/{}/{}/{}",
            state.static_url(&mime),
            author,
            post,
            filename
        );

        Ok(FileMetaJson {
            id,
            url,
            mime,
            extra,
        })
    }
}

fn parse_json(json: String) -> Result<Value, rusqlite::Error> {
    let Ok(json) = serde_json::from_str(&json) else {
        error!("Failed to parse, {}", json);
        return Err(rusqlite::Error::UnwindingPanic);
    };

    Ok(json)
}

#[derive(Debug, Clone, Serialize, Deserialize,TS)]
#[ts(export)]
pub struct TagJson(Tag);

impl TagJson {
    pub fn from_post(
        state: &AppState,
        tx: &Transaction,
        post: PostId,
    ) -> Result<Vec<Self>, rusqlite::Error> {
        let mut stmt = tx.prepare_cached(
            "SELECT * FROM tags
            JOIN post_tags ON post_tags.tag = tags.id
            WHERE post_tags.post = ?",
        )?;

        let mut rows = stmt.query([post])?;

        let mut data = Vec::new();
        while let Some(row) = rows.next()? {
            let tag = Tag::from_row(state, row)?;
            data.push(TagJson(tag));
        }

        Ok(data)
    }

    pub fn all(
        state: &AppState,
        tx: &Transaction,
    ) -> Result<Vec<Self>, rusqlite::Error> {
        let mut stmt = tx.prepare_cached("SELECT * FROM tags")?;
        let mut rows = stmt.query([])?;

        let mut data = Vec::new();
        while let Some(row) = rows.next()? {
            let tag = Tag::from_row(state, row)?;
            data.push(TagJson(tag));
        }

        Ok(data)
    }
}

impl FromRow for Tag {
    fn from_row(_state: &AppState, row: &Row) -> Result<Self, rusqlite::Error>
    where
        Self: Sized {
        let id: PostTagId = row.get(0)?;
        let name: String = row.get(1)?;

        Ok(Tag { id, name })
    }
}

impl Deref for TagJson {
    type Target = Tag;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}