pub mod author;
pub mod collection;
pub mod platform;
pub mod tag;

use std::hash::Hash;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use axum_extra::extract::Query;
use mini_moka::sync::Cache;
use post_archiver::{manager::PostArchiverManager, FileMetaId};
use rusqlite::{params, OptionalExtension, Row, ToSql};
use serde::{Deserialize, Serialize};

use crate::api::utils::into_thumb_url;

use super::{
    utils::{Pagination, PostMiniJson, WithThumb},
    AppState,
};

pub trait Category: Serialize + Sized {
    type Id: From<u32> + ToSql + Copy + Eq + Hash + Sync + Send + 'static;
    const TABLE_NAME: &'static str;
    const ORDER_BY: &'static str;
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error>;
    fn thumb(&self) -> Option<FileMetaId>;

    fn with_thumb_url(
        self,
        manager: &PostArchiverManager,
    ) -> Result<WithThumb<Self>, rusqlite::Error> {
        let Some(thumb) = self.thumb() else {
            return Ok(WithThumb {
                category: self,
                thumb: None,
            });
        };

        let thumb = manager.get_file_meta(&thumb)?;

        Ok(WithThumb {
            category: self,
            thumb: Some(thumb),
        })
    }
}

pub trait CategoryApiRouter: Category + Sized + 'static {
    const ROUTE_NAME: &'static str;

    fn wrap_category_route(router: Router<AppState>) -> Router<AppState> {
        router.route(
            &format!("/{}", Self::ROUTE_NAME),
            get(list_category_handler::<Self>),
        )
    }

    fn list(
        manager: &PostArchiverManager,
        pagination: Pagination,
        search: String,
    ) -> Result<Vec<WithThumb<Self>>, rusqlite::Error> {

        let (filter, params) = if search.is_empty() {
            ("", params![])
        } else {
            ("WHERE name LIKE concat('%',?,'%')", params![search])
        };

        let mut stmt = manager.conn().prepare_cached(&format!(
            "SELECT * FROM {} {} {} {}",
            Self::TABLE_NAME,
            filter,
            Self::ORDER_BY,
            pagination.to_sql(),
        ))?;

        let rows = stmt
            .query_map(params, Self::from_row)?
            .map(|row| row.and_then(|r| r.with_thumb_url(manager)));

        rows.collect()
    }

    fn total(state: &AppState, manager: &PostArchiverManager) -> Result<usize, rusqlite::Error> {
        if let Some(total) = state.caches.tables.get(&Self::TABLE_NAME) {
            return Ok(total);
        }

        let mut stmt = manager
            .conn()
            .prepare_cached(&format!("SELECT COUNT() FROM {}", Self::TABLE_NAME))?;

        let total = stmt.query_row([], |row| row.get(0))?;
        state.caches.tables.insert(Self::TABLE_NAME, total);
        Ok(total)
    }

    fn list_and_total(
        state: &AppState,
        manager: &PostArchiverManager,
        pagination: Pagination,
        search: String,
    ) -> Result<CategoryJson<Self>, rusqlite::Error> {
        let categories = Self::list(manager, pagination, search)?;
        let total = Self::total(state, manager)?;
        Ok(CategoryJson { total, categories })
    }
}

pub trait CategoryPostsApiRouter: CategoryApiRouter {
    const JOIN_RELATION: &'static str;
    const FILTER: &'static str;

    fn wrap_category_and_posts_route(router: Router<AppState>) -> Router<AppState> {
        Self::wrap_category_route(router).route(
            &format!("/{}/{{id}}", Self::ROUTE_NAME),
            get(list_category_posts_handler::<Self>),
        )
    }

    fn list_posts(
        manager: &PostArchiverManager,
        pagination: Pagination,
        id: Self::Id,
    ) -> Result<Vec<PostMiniJson>, rusqlite::Error> {
        let mut stmt = manager
            .conn()
            .prepare_cached(&format!(
                "SELECT posts.id id, posts.title title, posts.updated updated, file_metas.filename thumb FROM posts JOIN file_metas ON posts.thumb = file_metas.id {} WHERE {} = ? ORDER BY posts.updated DESC {}",
                Self::JOIN_RELATION,
                Self::FILTER,
                pagination.to_sql(),
            ))?;

        let rows = stmt.query_map([id], |row| {
            let id = row.get("id")?;
            let filename: Option<String> = row.get("thumb")?;
            Ok(PostMiniJson {
                id,
                title: row.get("title")?,
                updated: row.get("updated")?,
                thumb: filename.map(|f| into_thumb_url(id, f)),
            })
        })?;

        rows.collect()
    }

    fn total_post(
        state: &AppState,
        manager: &PostArchiverManager,
        id: Self::Id,
    ) -> Result<usize, rusqlite::Error> {
        if let Some(total) = Self::get_post_cache(state, id) {
            return Ok(total);
        }

        let mut stmt = manager.conn().prepare_cached(&format!(
            "SELECT COUNT() FROM posts {} WHERE {} = ?",
            Self::JOIN_RELATION,
            Self::FILTER,
        ))?;

        let total = stmt.query_row([id], |row| row.get(0))?;
        Self::set_post_cache(state, id, total);

        Ok(total)
    }

    fn post_cache(state: &AppState) -> &Cache<Self::Id, usize>;

    fn get_post_cache(state: &AppState, id: Self::Id) -> Option<usize> {
        Self::post_cache(state).get(&id)
    }
    fn set_post_cache(state: &AppState, id: Self::Id, total: usize) {
        Self::post_cache(state).insert(id, total);
    }

    fn get(
        manager: &PostArchiverManager,
        id: Self::Id,
    ) -> Result<Option<WithThumb<Self>>, rusqlite::Error> {
        let mut stmt = manager
            .conn()
            .prepare_cached(&format!("SELECT * FROM {} WHERE id = ?", Self::TABLE_NAME))?;

        stmt.query_row([id], Self::from_row)
            .optional()?
            .map(|c| c.with_thumb_url(manager))
            .transpose()
    }

    fn get_category_and_posts(
        state: &AppState,
        manager: &PostArchiverManager,
        id: Self::Id,
        pagination: Pagination,
    ) -> Result<Option<CategoryPostsJson<Self>>, rusqlite::Error> {
        let Some(category) = Self::get(manager, id)? else {
            return Ok(None);
        };
        let posts = Self::list_posts(manager, pagination, id)?;
        let total = Self::total_post(state, manager, id)?;
        Ok(Some(CategoryPostsJson {
            category,
            posts,
            total,
        }))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Filter {
    #[serde(default)]
    pub search: String,
}

async fn list_category_handler<T: CategoryApiRouter>(
    Query(filter): Query<Filter>,
    Query(pagination): Query<Pagination>,
    State(state): State<AppState>,
) -> Result<Json<CategoryJson<T>>, StatusCode> {
    let manager = &state.manager();
    T::list_and_total(&state, manager, pagination, filter.search)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .map(Json::from)
}

async fn list_category_posts_handler<T: CategoryPostsApiRouter>(
    Path(id): Path<u32>,
    Query(pagination): Query<Pagination>,
    State(state): State<AppState>,
) -> Result<Json<CategoryPostsJson<T>>, StatusCode> {
    let manager = &state.manager();
    let id: T::Id = id.into();

    T::get_category_and_posts(&state, manager, id, pagination)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)
        .map(Json::from)
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoryJson<T: Category> {
    categories: Vec<WithThumb<T>>,
    total: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoryPostsJson<T: Category> {
    #[serde(flatten)]
    category: WithThumb<T>,
    total: usize,
    posts: Vec<PostMiniJson>,
}
