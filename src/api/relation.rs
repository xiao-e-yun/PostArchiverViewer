use std::{collections::HashSet, fmt::Debug, hash::Hash};

use post_archiver::{
    manager::PostArchiverManager, Author, AuthorId, Collection, CollectionId, FileMeta, FileMetaId,
    Platform, PlatformId, Tag, TagId,
};
use rusqlite::Connection;
use serde::Serialize;
use ts_rs::TS;

use super::category::Category;

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct WithRelation<T: Debug> {
    #[serde(flatten)]
    pub inner: T,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<Author>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub collections: Vec<Collection>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub platforms: Vec<Platform>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub file_metas: Vec<FileMeta>,
}

impl<T: Debug + RequireRelations> WithRelation<T> {
    pub fn new(manager: &PostArchiverManager, inner: T) -> Result<Self, rusqlite::Error> {
        let conn = manager.conn();

        let authors = Author::query(conn, inner.authors())?;
        let collections = Collection::query(conn, inner.collections())?;
        let tags = Tag::query(conn, inner.tags())?;
        let platforms = Platform::query(
            conn,
            inner
                .platforms()
                .into_iter()
                .chain(tags.iter().flat_map(|t| t.platform)),
        )?;
        let file_metas = FileMeta::query(
            conn,
            inner
                .file_metas()
                .into_iter()
                .chain(authors.iter().flat_map(|a| a.thumb))
                .chain(collections.iter().flat_map(|c| c.thumb)),
        )?;

        Ok(Self {
            inner,
            authors,
            collections,
            platforms,
            tags,
            file_metas,
        })
    }
}

pub trait RequireRelations {
    fn authors(&self) -> Vec<AuthorId> {
        vec![]
    }
    fn collections(&self) -> Vec<CollectionId> {
        vec![]
    }
    fn platforms(&self) -> Vec<PlatformId> {
        vec![]
    }
    fn tags(&self) -> Vec<TagId> {
        vec![]
    }
    fn file_metas(&self) -> Vec<FileMetaId> {
        vec![]
    }
}

impl<T: RequireRelations> RequireRelations for Vec<T> {
    fn authors(&self) -> Vec<AuthorId> {
        self.iter().flat_map(|item| item.authors()).collect()
    }
    fn collections(&self) -> Vec<CollectionId> {
        self.iter().flat_map(|item| item.collections()).collect()
    }
    fn platforms(&self) -> Vec<PlatformId> {
        self.iter().flat_map(|item| item.platforms()).collect()
    }
    fn tags(&self) -> Vec<TagId> {
        self.iter().flat_map(|item| item.tags()).collect()
    }
    fn file_metas(&self) -> Vec<FileMetaId> {
        self.iter().flat_map(|item| item.file_metas()).collect()
    }
}

pub trait RelationTarget: Serialize + Sized {
    type Id: Serialize + Debug + Eq + Hash;
    const TABLE_NAME: &'static str;

    fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error>;

    fn query(
        conn: &Connection,
        ids: impl IntoIterator<Item = Self::Id>,
    ) -> Result<Vec<Self>, rusqlite::Error> {
        let ids: HashSet<Self::Id> = ids.into_iter().collect();
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let mut stmt = conn.prepare_cached(&format!(
            "SELECT * FROM {} WHERE id IN (SELECT value FROM json_each(?))",
            Self::TABLE_NAME
        ))?;

        let rows = stmt.query_map([serde_json::to_string(&ids).unwrap()], |row| {
            Self::from_row(row)
        })?;

        rows.collect()
    }
}

impl<T: Category> RelationTarget for T {
    type Id = T::Id;
    const TABLE_NAME: &'static str = T::TABLE_NAME;

    fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
        T::from_row(row)
    }
}

impl RelationTarget for FileMeta {
    type Id = FileMetaId;
    const TABLE_NAME: &'static str = "file_metas";

    fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
        FileMeta::from_row(row)
    }
}
