pub mod author;
pub mod collection;
pub mod platform;
pub mod tag;

use std::{fmt::Debug, hash::Hash};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use axum_extra::extract::Query;
use mini_moka::sync::Cache;
use post_archiver::{manager::PostArchiverManager};
use rusqlite::{params, OptionalExtension, Row, ToSql};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    relation::{RequireRelations, WithRelations},
    utils::{ListResponse, Pagination, PostListResponse, PostPreview},
    AppState,
};

pub trait Category: RequireRelations + Serialize + Debug + TS + Sized {
    type Id: From<u32> + Debug + Serialize + ToSql + Copy + Eq + Hash + Sync + Send + 'static;
    const TABLE_NAME: &'static str;
    const ORDER_BY: &'static str;
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error>;
}

pub trait CategoryApiRouter: Category + 'static {
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
    ) -> Result<Vec<Self>, rusqlite::Error> {
        let [limit, offset] = pagination.params();
        let (filter, params) = if search.is_empty() {
            ("", params![limit, offset])
        } else {
            ("WHERE name LIKE concat('%',?,'%')", params![search, limit, offset])
        };

        let mut stmt = manager.conn().prepare_cached(&format!(
            "SELECT * FROM {} {} {} LIMIT ? OFFSET ?",
            Self::TABLE_NAME,
            filter,
            Self::ORDER_BY,
        ))?;

        let list = stmt
            .query_map(params, Self::from_row)?;

        list.collect()
    }

    fn total(
        state: &AppState,
        manager: &PostArchiverManager,
        search: String,
    ) -> Result<usize, rusqlite::Error> {
        if !search.is_empty() {
            let mut stmt = manager.conn().prepare_cached(&format!(
                "SELECT COUNT() FROM {} WHERE name LIKE concat('%',?,'%')",
                Self::TABLE_NAME
            ))?;
            return stmt.query_row([search], |row| row.get(0));
        }

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
}

pub trait CategoryPostsApiRouter: CategoryApiRouter {
    const JOIN_RELATION: &'static str;
    const FILTER: &'static str;

    fn wrap_category_and_posts_route(router: Router<AppState>) -> Router<AppState> {
        Self::wrap_category_route(router)
            .route(
                &format!("/{}/{{id}}", Self::ROUTE_NAME),
                get(get_category_handler::<Self>),
            )
            .route(
                &format!("/{}/{{id}}/posts", Self::ROUTE_NAME),
                get(list_category_posts_handler::<Self>),
            )
    }

    fn list_posts(
        manager: &PostArchiverManager,
        pagination: Pagination,
        id: Self::Id,
    ) -> Result<Vec<PostPreview>, rusqlite::Error> {
        let mut stmt = manager.conn()
            .prepare_cached(&format!(
                "SELECT posts.* FROM posts {} WHERE {} = ? ORDER BY posts.updated DESC LIMIT ? OFFSET ?",
                Self::JOIN_RELATION,
                Self::FILTER,
            ))?;

        let [limit, offset] = pagination.params();
        let rows = stmt.query_map(params![id, limit, offset], PostPreview::from_row)?;

        rows.collect::<Result<Vec<PostPreview>, _>>()
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
    ) -> Result<Option<WithRelations<Self>>, rusqlite::Error> {
        let mut stmt = manager
            .conn()
            .prepare_cached(&format!("SELECT * FROM {} WHERE id = ?", Self::TABLE_NAME))?;

        stmt.query_row([id], Self::from_row)
            .optional()?
            .map(|c| WithRelations::new(manager, c))
            .transpose()
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
) -> Result<Json<WithRelations<ListResponse<T>>>, StatusCode> {
    let manager = &state.manager();
    let list = T::list(manager, pagination, filter.search.clone())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let total =
        T::total(&state, manager, filter.search).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    WithRelations::new(
        manager,
        ListResponse { list, total },
    )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .map(Json::from)
}

async fn get_category_handler<T: CategoryPostsApiRouter>(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> Result<Json<WithRelations<T>>, StatusCode> {
    let manager = &state.manager();
    let id: T::Id = id.into();

    T::get(manager, id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)
        .map(Json::from)
}

async fn list_category_posts_handler<T: CategoryPostsApiRouter>(
    Path(id): Path<u32>,
    Query(pagination): Query<Pagination>,
    State(state): State<AppState>,
) -> Result<Json<WithRelations<PostListResponse>>, StatusCode> {
    let manager = &state.manager();
    let id: T::Id = id.into();

    let list = T::list_posts(manager, pagination, id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total = T::total_post(&state, manager, id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = PostListResponse {
        list,
        total,
    };

    WithRelations::new(manager, response)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .map(Json::from)
}

