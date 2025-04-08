use std::mem;

use chrono::{DateTime, Utc};
use post_archiver::{
    manager::PostArchiverManager,
    utils::{author::GetAuthor, file_meta::GetFileMeta, post::GetPost},
    Author, AuthorId, Comment, Content, FileMeta, FileMetaId, Link, Post, PostId, Tag,
};
use rusqlite::Row;
use serde::{Deserialize, Serialize};
use tracing::error;
use ts_rs::TS;

pub trait FromRow {
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error>
    where
        Self: Sized;
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AuthorJson {
    pub id: AuthorId,
    pub name: String,
    pub links: Vec<Link>,
    pub thumb: Option<FileMeta>,
    pub updated: DateTime<Utc>,
}

impl AuthorJson {
    pub fn resolve(manager: &PostArchiverManager, author: Author) -> Result<Self, rusqlite::Error> {
        let thumb = author.thumb.map(|id| id.file_meta(manager)).transpose()?;

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
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
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

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PostJson {
    pub id: PostId,
    pub author: Author,
    pub source: Option<String>,
    pub title: String,
    pub content: Vec<ContentJson>,
    pub thumb: Option<FileMeta>,
    pub comments: Vec<Comment>,
    pub updated: DateTime<Utc>,
    pub published: DateTime<Utc>,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(untagged)]
#[ts(export)]
pub enum ContentJson {
    Text(String),
    File(FileMeta),
}

impl PostJson {
    pub fn resolve(
        manager: &PostArchiverManager,
        mut post: Post,
    ) -> Result<Option<Self>, rusqlite::Error> {
        let author = post.author.author(manager)?;
        let content = mem::take(&mut post.content)
            .into_iter()
            .map(|content| {
                Ok(match content {
                    Content::Text(text) => ContentJson::Text(text.clone()),
                    Content::File(id) => ContentJson::File(id.file_meta(manager)?),
                })
            })
            .collect::<Result<Vec<ContentJson>, rusqlite::Error>>()?;

        let thumb = post
            .thumb
            .map(|thumb| thumb.file_meta(manager))
            .transpose()?;

        let tags = post.post_tags(manager)?;

        Ok(Some(PostJson {
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
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PostMiniJson {
    pub id: PostId,
    pub author: Author,
    pub title: String,
    pub thumb: Option<FileMeta>,
    pub updated: DateTime<Utc>,
}

impl PostMiniJson {
    pub fn resolve(
        manager: &PostArchiverManager,
        post: Post,
    ) -> Result<Option<Self>, rusqlite::Error> {
        let author = post.author.author(manager)?;

        let thumb = post
            .thumb
            .map(|thumb| thumb.file_meta(manager))
            .transpose()?;

        Ok(Some(PostMiniJson {
            id: post.id,
            author,
            title: post.title,
            thumb,
            updated: post.updated,
        }))
    }
}

impl FromRow for Post {
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error>
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

pub fn list_tags(manager: &PostArchiverManager) -> Result<Vec<Tag>, rusqlite::Error> {
    let mut stmt = manager.conn().prepare_cached("SELECT * FROM tags")?;
    let mut rows = stmt.query([])?;

    let mut data = Vec::new();
    while let Some(row) = rows.next()? {
        data.push(Tag {
            id: row.get("id")?,
            name: row.get("name")?,
        });
    }

    Ok(data)
}
