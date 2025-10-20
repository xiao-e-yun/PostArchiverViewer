use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use axum_extra::extract::Query;
use cached::Cached;
use post_archiver::{AuthorId, CollectionId, PlatformId, TagId};
use rusqlite::{params_from_iter, CachedStatement, Connection};
use serde::{Deserialize, Serialize};

#[cfg(feature = "full-text-search")]
use crate::config::Config;
#[cfg(feature = "full-text-search")]
use tracing::info;

use super::{
    post::get_post_handler,
    relation::WithRelations,
    utils::{ListResponse, Pagination, PostPreview},
    AppState,
};

pub fn wrap_posts_route(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/posts", get(list_posts_handler))
        .route("/posts/{id}", get(get_post_handler))
}

#[cfg(feature = "full-text-search")]
use post_archiver::manager::PostArchiverManager;

#[cfg(feature = "full-text-search")]
pub fn sync_text_search(config: &Config, manager: &mut PostArchiverManager) -> bool {
    let old_status = manager
        .get_feature("PostArchiverViewer:SearchFullText")
        .unwrap_or(0)
        != 0;

    let status = config.futures.full_text_search.unwrap_or(old_status);
    let changed = old_status != status;

    info!(
        "search-full-text: {} {}",
        if status { "enabled" } else { "disabled" },
        if changed { "(changed)" } else { "" }
    );

    if changed {
        let transaction = manager.transaction().unwrap();
        transaction.set_feature("PostArchiverViewer:SearchFullText", status as i64);

        let conn = transaction.conn();
        if status {
            info!("creating search table");
            conn.execute_batch(
                        "CREATE VIRTUAL TABLE _posts_fts USING fts5(title, content, content=posts, content_rowid=id, tokenize = 'simple');"
                    )
                    .unwrap();
        } else {
            info!("delete search table");
            conn.execute_batch("DROP TABLE _posts_fts;").unwrap();
        }
        transaction.commit().unwrap();

        info!("cleanup database");
        manager.conn().execute_batch("VACUUM;").unwrap();
    }

    if status {
        info!("rebuilt full-text search");
        manager
            .conn()
            .execute_batch("INSERT INTO _posts_fts(_posts_fts) VALUES('rebuild');")
            .unwrap();
    }

    status
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "lowercase")]
pub enum PostOrderBy {
    Id,
    #[default]
    Updated,
    Random,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Default)]
pub struct SearchQuery {
    #[serde(default)]
    search: String,
    #[serde(default)]
    tags: Vec<TagId>,
    #[serde(default)]
    collections: Vec<CollectionId>,
    #[serde(default)]
    authors: Vec<AuthorId>,
    #[serde(default)]
    platforms: Vec<PlatformId>,
    #[serde(default)]
    order_by: PostOrderBy,
}

type SearchContext = (
    Vec<&'static str>,
    Vec<&'static str>,
    Vec<&'static str>,
    Vec<String>,
);
pub async fn list_posts_handler(
    Query(pagination): Query<Pagination>,
    Query(query): Query<SearchQuery>,
    State(state): State<AppState>,
) -> Result<Json<WithRelations<ListResponse<PostPreview>>>, StatusCode> {
    let manager = state.manager();
    let conn = manager.conn();

    let mut context: SearchContext = (vec![], vec![], vec![], vec![]);
    
    bind_search(&mut context, state.full_text_search(), &query.search);
    bind_relation(
        &mut context,
        &query.authors,
        &query.tags,
        &query.collections,
        &query.platforms,
    );

    let mut stmt = prepare_search(&context, query.order_by, conn).unwrap();

    let pagination = pagination.params().map(|p| p.to_string());
    let params = params_from_iter(context.3.iter().chain(pagination.iter()));

    let rows = stmt
        .query_map(params.clone(), PostPreview::from_row)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let list = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let params = params_from_iter(context.3.iter());
    let mut cache = state.caches.posts.lock().unwrap();
    let total = match cache.cache_get(&query) {
        Some(cached) => *cached,
        None => {
            let mut stmt = prepare_search_total(&context, conn).unwrap();

            let total = match stmt.query_row(params, |row| row.get(0)) {
                Ok(total) => total,
                Err(rusqlite::Error::QueryReturnedNoRows) => 0,
                Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            };

            cache.cache_set(query, total);

            total
        }
    };

    WithRelations::new(&manager, ListResponse { list, total })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .map(Json::from)
}

fn prepare_search<'a>(
    (joins, filters, havings, _params): &SearchContext,
    order_by: PostOrderBy,
    connection: &'a Connection,
) -> Result<CachedStatement<'a>, rusqlite::Error> {
    let joins = joins.join(" ");

    let filters = if filters.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", filters.join(" AND "))
    };

    let havings = if havings.is_empty() {
        String::new()
    } else {
        format!("GROUP BY posts.id HAVING {}", havings.join(" AND "))
    };

    let order_by = match order_by {
        PostOrderBy::Id => "posts.id DESC",
        PostOrderBy::Updated => "posts.updated DESC",
        PostOrderBy::Random => "RANDOM()",
    };

    let sql  = format!(
        "SELECT id, title, updated, thumb FROM posts {joins} {filters} {havings} ORDER BY {order_by} LIMIT ? OFFSET ?"
    );

    connection.prepare_cached(&sql)
}

fn prepare_search_total<'a>(
    (joins, filters, havings, _params): &SearchContext,
    connection: &'a Connection,
) -> Result<CachedStatement<'a>, rusqlite::Error> {
    let joins = joins.join(" ");

    let filters = if filters.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", filters.join(" AND "))
    };

    let havings = if havings.is_empty() {
        String::new()
    } else {
        format!("GROUP BY posts.id HAVING {}", havings.join(" AND "))
    };

    let sql = format!("SELECT count() FROM (SELECT 0 FROM posts {joins} {filters} {havings})");

    connection.prepare_cached(&sql)
}

fn bind_relation(
    (joins, filters, havings, params): &mut SearchContext,
    authors: &[AuthorId],
    tags: &[TagId],
    collections: &[CollectionId],
    platforms: &[PlatformId],
) {
    let mut havings_with_params = vec![];

    if !authors.is_empty() {
        joins.push("JOIN author_posts ON posts.id = author_posts.post AND author_posts.author IN (SELECT value FROM json_each(?))");
        params.push(serde_json::to_string(&authors).unwrap());
        if authors.len() > 1 {
            havings_with_params.push((
                "COUNT(DISTINCT author_posts.author) == CAST(? AS INTEGER)",
                authors.len(),
            ));
        }
    }

    if !tags.is_empty() {
        joins.push("JOIN post_tags ON posts.id = post_tags.post AND post_tags.tag IN (SELECT value FROM json_each(?))");
        params.push(serde_json::to_string(&tags).unwrap());
        if tags.len() > 1 {
            havings_with_params.push((
                "COUNT(DISTINCT post_tags.tag) == CAST(? AS INTEGER)",
                tags.len(),
            ));
        }
    }

    if !collections.is_empty() {
        joins.push("JOIN collection_posts ON posts.id = collection_posts.post AND collection_posts.collection IN (SELECT value FROM json_each(?))");
        params.push(serde_json::to_string(&collections).unwrap());
        if collections.len() > 1 {
            havings_with_params.push((
                "COUNT(DISTINCT collection_posts.collection) == CAST(? AS INTEGER)",
                collections.len(),
            ));
        }
    }

    if !platforms.is_empty() {
        filters.push("posts.platform IN (SELECT value FROM json_each(?))");
        params.push(serde_json::to_string(&platforms).unwrap());
    }

    if !havings_with_params.is_empty() {
        for (condition, count) in havings_with_params {
            havings.push(condition);
            params.push(count.to_string());
        }
    }
}

fn bind_search(
    (joins, filters, _havings, params): &mut SearchContext,
    full_text_search: bool,
    search: &str,
) {
    if search.is_empty() {
        return;
    }

    params.push(search.to_string());
    match full_text_search {
        true => {
            joins.push("JOIN _posts_fts ON posts.id = _posts_fts.rowid");
            filters.push("_posts_fts MATCH ?");
        }
        false => filters.push("posts.title LIKE concat('%', ?, '%')"),
    };
}
