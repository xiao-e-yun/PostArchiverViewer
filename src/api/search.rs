use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::Query;
use post_archiver::{AuthorId, CollectionId, PlatformId, TagId};
use rusqlite::{params_from_iter, CachedStatement, Connection};
use serde::{Deserialize, Serialize};

#[cfg(feature = "full-text-search")]
use crate::config::Config;
#[cfg(feature = "full-text-search")]
use tracing::info;

use super::{
    relation::WithRelations,
    utils::{ListResponse, Pagination, PostPreview},
    AppState,
};

#[cfg(feature = "full-text-search")]
use post_archiver::manager::PostArchiverManager;

#[cfg(feature = "full-text-search")]
pub fn sync_search_api(config: &Config, manager: &mut PostArchiverManager) -> bool {
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
}

type SearchContext = (Vec<&'static str>, Vec<&'static str>, Vec<String>);
pub async fn get_search_api(
    Query(pagination): Query<Pagination>,
    Query(query): Query<SearchQuery>,
    State(state): State<AppState>,
) -> Result<Json<WithRelations<ListResponse<PostPreview>>>, StatusCode> {
    let manager = state.manager();
    let conn = manager.conn();

    let mut context: SearchContext = (vec![], vec![], vec![]);
    bind_search(&mut context, state.full_text_search(), &query.search);
    bind_relation(
        &mut context,
        &query.authors,
        &query.tags,
        &query.collections,
        &query.platforms,
    );

    let mut stmt = prepare_search(&context, conn).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let pagination = pagination.params().map(|p| p.to_string());
    let params = params_from_iter(context.2.iter().chain(pagination.iter()));

    let rows = stmt
        .query_map(params.clone(), PostPreview::from_row)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let list = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let params = params_from_iter(context.2.iter());
    let total = match state.caches.search.get(&query) {
        Some(cached) => cached,
        None => {
            let mut stmt = prepare_search_total(&context, conn)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let total = stmt
                .query_row(params, |row| row.get(0))
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            state.caches.search.insert(query, total);

            total
        }
    };

    WithRelations::new(&manager, ListResponse { list, total })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .map(Json::from)
}

fn prepare_search<'a>(
    (joins, filters, _params): &SearchContext,
    connection: &'a Connection,
) -> Result<CachedStatement<'a>, rusqlite::Error> {
    let joins = joins.join(" ");

    let filters = if filters.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", filters.join(" AND "))
    };

    let sql  = format!(
        "SELECT id, title, updated, thumb FROM posts {joins} {filters} ORDER BY posts.updated DESC LIMIT ? OFFSET ?"
    );

    connection.prepare_cached(&sql)
}

fn prepare_search_total<'a>(
    (joins, filters, _params): &SearchContext,
    connection: &'a Connection,
) -> Result<CachedStatement<'a>, rusqlite::Error> {
    let joins = joins.join(" ");

    let filters = if filters.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", filters.join(" AND "))
    };

    let sql = format!("SELECT count() FROM posts {joins} {filters}");

    connection.prepare_cached(&sql)
}

fn bind_relation(
    (joins, filters, params): &mut SearchContext,
    authors: &[AuthorId],
    tags: &[TagId],
    collections: &[CollectionId],
    platforms: &[PlatformId],
) {
    if !authors.is_empty() {
        joins.push("JOIN author_posts ON posts.id = author_posts.post");
        filters.push("author_posts.author IN (SELECT value FROM json_each(?))");
        params.push(serde_json::to_string(&authors).unwrap());
    }

    if !tags.is_empty() {
        joins.push("JOIN post_tags ON posts.id = post_tags.post");
        filters.push("post_tags.tag IN (SELECT value FROM json_each(?))");
        params.push(serde_json::to_string(&tags).unwrap());
    }

    if !collections.is_empty() {
        joins.push("JOIN collection_posts ON posts.id = collection_posts.post");
        filters.push("collection_posts.collection IN (SELECT value FROM json_each(?))");
        params.push(serde_json::to_string(&collections).unwrap());
    }

    if !platforms.is_empty() {
        filters.push("posts.platform IN (SELECT value FROM json_each(?))");
        params.push(serde_json::to_string(&platforms).unwrap());
    }
}

fn bind_search((joins, filters, params): &mut SearchContext, full_text_search: bool, search: &str) {
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
