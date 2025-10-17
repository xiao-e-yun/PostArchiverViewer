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
use cached::Cached;
use post_archiver::manager::PostArchiverManager;
use rusqlite::{params, OptionalExtension, Row, ToSql};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    relation::{RequireRelations, WithRelations},
    utils::{ListResponse, Pagination},
    AppState,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CategoryOrderBy {
    Id,
    Name,
    Updated,
    Random,
}

pub trait Category: RequireRelations + Serialize + Debug + TS + Sized + 'static {
    type Id: From<u32> + Debug + Serialize + ToSql + Copy + Eq + Hash + Sync + Send + 'static;
    const TABLE_NAME: &'static str;
    const DEFAULT_ORDER_BY: CategoryOrderBy = CategoryOrderBy::Name;
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error>;

    fn wrap_category_route(router: Router<AppState>) -> Router<AppState> {
        router
            .route(
                &format!("/{}", Self::TABLE_NAME),
                get(list_category_handler::<Self>),
            )
            .route(
                &format!("/{}/{{id}}", Self::TABLE_NAME),
                get(get_category_handler::<Self>),
            )
    }

    fn list(
        manager: &PostArchiverManager,
        pagination: Pagination,
        search: String,
        order_by: Option<CategoryOrderBy>,
    ) -> Result<Vec<Self>, rusqlite::Error> {
        let [limit, offset] = pagination.params();
        let (filter, params) = if search.is_empty() {
            ("", params![limit, offset])
        } else {
            (
                "WHERE name LIKE concat('%',?,'%')",
                params![search, limit, offset],
            )
        };

        let mut stmt = manager.conn().prepare_cached(&format!(
            "SELECT * FROM {} {} {} LIMIT ? OFFSET ?",
            Self::TABLE_NAME,
            filter,
            match (order_by.unwrap_or(Self::DEFAULT_ORDER_BY), Self::TABLE_NAME) {
                (CategoryOrderBy::Id, _) => "ORDER BY id DESC",
                (CategoryOrderBy::Name, _) => "ORDER BY name ASC",
                (CategoryOrderBy::Updated, "author") => "ORDER BY updated DESC",
                (CategoryOrderBy::Updated, _) => "ORDER BY id DESC", // Fallback to id
                (CategoryOrderBy::Random, _) => "ORDER BY RANDOM()",
            }
        ))?;

        let list = stmt.query_map(params, Self::from_row)?;

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

        let mut cache = state
            .caches
            .tables
            .lock()
            .unwrap();

        if let Some(cached) = cache.cache_get(&Self::TABLE_NAME)
        {
            return Ok(*cached);
        }

        let mut stmt = manager
            .conn()
            .prepare_cached(&format!("SELECT COUNT() FROM {}", Self::TABLE_NAME))?;

        let total = stmt.query_row([], |row| row.get(0))?;
        cache.cache_set(Self::TABLE_NAME, total);

        Ok(total)
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

#[derive(Debug, Deserialize)]
pub struct Filter {
    #[serde(default)]
    pub search: String,
    pub order_by: Option<CategoryOrderBy>,
}

async fn list_category_handler<T: Category>(
    Query(filter): Query<Filter>,
    Query(pagination): Query<Pagination>,
    State(state): State<AppState>,
) -> Result<Json<WithRelations<ListResponse<T>>>, StatusCode> {
    let manager = &state.manager();
    let list = T::list(manager, pagination, filter.search.clone(), filter.order_by)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let total =
        T::total(&state, manager, filter.search).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    WithRelations::new(manager, ListResponse { list, total })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .map(Json::from)
}

async fn get_category_handler<T: Category>(
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
