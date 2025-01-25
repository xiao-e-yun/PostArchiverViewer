use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use post_archiver::{AuthorId, Comment, Content, FileMetaId, PostId, Tag};
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone)]
pub struct AppState {
    conn: Arc<Mutex<Connection>>,
    static_server_url: Option<String>,
}

impl AppState {
    pub fn conn(&self) -> std::sync::MutexGuard<Connection> {
        self.conn.lock().unwrap()
    }
    pub fn static_url(&self, mime: &str) -> String {
        match &self.static_server_url {
            Some(url) => url.clone(),
            None => if mime.starts_with("image/") {
                "/images"
            } else {
                "/resource"
            }
            .to_string(),
        }
    }
}

#[derive(Serialize)]
pub struct APIResponse<T> {
    data: T,
}

impl<T: Serialize> IntoResponse for APIResponse<T> {
    fn into_response(self) -> axum::response::Response {
        Json(self.data).into_response()
    }
}

pub fn get_api_router(archiver_path: &PathBuf, static_server_url: Option<String>) -> Router {
    let conn = Connection::open(archiver_path.join("post-archiver.db")).unwrap();
    let conn = Arc::new(Mutex::new(conn));
    let state = AppState {
        static_server_url,
        conn,
    };

    Router::new()
        .route("/authors", get(get_authors_api))
        .route("/posts", get(get_posts_api))
        .route("/post", get(get_post_api))
        .route("/post/tags", get(get_post_tags_api))
        .route("/files", get(get_file_metas_api))
        .route("/tags", get(get_tags_api))
        .route("/info", get(get_info_api))
        .fallback(StatusCode::NOT_FOUND)
        .with_state(state)
}

async fn get_authors_api(
    State(state): State<AppState>,
) -> Result<APIResponse<Vec<Value>>, StatusCode> {
    let conn = state.conn();
    let mut stmt = conn
        .prepare_cached("SELECT * FROM authors ORDER BY updated DESC")
        .unwrap();
    let mut rows = stmt.query([]).unwrap();

    let mut data = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        let id = AuthorId(row.get_unwrap(0));
        let name: String = row.get_unwrap(1);
        let links: Value = serde_json::from_str(&row.get_unwrap::<_, String>(2)).unwrap();
        let thumb = row.get_unwrap::<_, Option<u32>>(3).map(FileMetaId);
        let updated: String = row.get_unwrap(4);

        let thumb = thumb
            .map(|id| get_file_meta(&state, &conn, id))
            .transpose()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        data.push(json!({
            "id":id,
            "links":links,
            "name":name,
            "thumb":thumb,
            "updated":updated,
        }))
    }

    Ok(APIResponse { data })
}

async fn get_file_metas_api(State(state): State<AppState>) -> APIResponse<Vec<Value>> {
    let conn = state.conn();
    let mut stmt = conn
        .prepare_cached("SELECT * FROM file_metas ORDER BY updated DESC")
        .unwrap();
    let mut rows = stmt.query([]).unwrap();

    let mut data = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        let id = FileMetaId(row.get_unwrap(0));
        let filename = row.get_unwrap::<_, String>(1);
        let author = AuthorId(row.get_unwrap(2));
        let post = PostId(row.get_unwrap(3));
        let mime = row.get_unwrap::<_, String>(4);
        let extra: Value = serde_json::from_str(&row.get_unwrap::<_, String>(5)).unwrap();
        let url = format!(
            "{}/{}/{}/{}",
            state.static_url(&mime),
            author,
            post,
            filename
        );

        data.push(json!({
            "id":id,
            "filename":filename,
            "author":author,
            "post":post,
            "mime":mime,
            "extra":extra,
            "url":url,
        }));
    }

    APIResponse { data }
}

async fn get_info_api(State(state): State<AppState>) -> APIResponse<Value> {
    let conn = state.conn();

    let count_authors: u32 = conn
        .query_row("SELECT COUNT(*) FROM authors", [], |row| row.get(0))
        .unwrap();
    let count_posts: u32 = conn
        .query_row("SELECT COUNT(*) FROM posts", [], |row| row.get(0))
        .unwrap();
    let count_files: u32 = conn
        .query_row("SELECT COUNT(*) FROM file_metas", [], |row| row.get(0))
        .unwrap();

    APIResponse {
        data: json!({
            "authors":count_authors,
            "posts":count_posts,
            "files":count_files,
        }),
    }
}

#[derive(Debug, Deserialize)]
pub struct PostsQuery {
    author: u32,
}

async fn get_posts_api(
    Query(query): Query<PostsQuery>,
    State(state): State<AppState>,
) -> Result<APIResponse<Vec<Value>>, StatusCode> {
    let conn = state.conn();
    let mut stmt = conn
        .prepare_cached("SELECT id, author, title, thumb, updated FROM posts WHERE author = ? ORDER BY updated DESC")
        .unwrap();
    let mut rows = stmt.query([query.author]).unwrap();

    let mut data = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        let id = PostId(row.get_unwrap(0));
        let author = AuthorId(row.get_unwrap(1));
        let title: String = row.get_unwrap(2);
        let thumb = row.get_unwrap::<_, Option<u32>>(3).map(FileMetaId);
        let updated: String = row.get_unwrap(4);

        let thumb = thumb
            .map(|id| get_file_meta(&state, &conn, id))
            .transpose()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        data.push(json!({
            "id":id,
            "author":author,
            "title":title,
            "thumb":thumb,
            "updated":updated,
        }));
    }

    Ok(APIResponse { data })
}

#[derive(Debug, Deserialize)]
pub struct PostQuery {
    post: u32,
}

async fn get_post_api(
    Query(query): Query<PostQuery>,
    State(state): State<AppState>,
) -> Result<APIResponse<Value>, StatusCode> {
    let conn = state.conn();
    let Ok(mut stmt) = conn.prepare_cached("SELECT * FROM posts WHERE id = ?") else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let data = match stmt
        .query_row([query.post], |row| {
            let id = PostId(row.get_unwrap(0));
            let author = AuthorId(row.get_unwrap(1));
            let source: String = row.get_unwrap(2);
            let title: String = row.get_unwrap(3);
            let content: Vec<Content> =
                serde_json::from_str(&row.get_unwrap::<_, String>(4)).unwrap_or_default();
            let thumb = row.get_unwrap::<_, Option<u32>>(5).map(FileMetaId);
            let comments: Vec<Comment> =
                serde_json::from_str(&row.get_unwrap::<_, String>(6)).unwrap_or_default();
            let updated: String = row.get_unwrap(7);
            let published: String = row.get_unwrap(8);

            let mut parsed_content = vec![];
            for c in content {
                parsed_content.push(match c {
                    Content::Text(text) => json!(text),
                    Content::File(id) => json!(get_file_meta(&state, &conn, id)?),
                })
            }

            let thumb = thumb
                .map(|id| get_file_meta(&state, &conn, id))
                .transpose()?;

            Ok(json!({
                "id":id,
                "author":author,
                "source":source,
                "title":title,
                "content":parsed_content,
                "thumb":thumb,
                "comments":comments,
                "updated":updated,
                "published":published,
            }))
        })
        .optional()
    {
        Ok(Some(data)) => data,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    Ok(APIResponse { data })
}

async fn get_post_tags_api(
    Query(query): Query<PostQuery>,
    State(state): State<AppState>,
) -> Result<APIResponse<Vec<Tag>>, StatusCode> {
    let conn = state.conn();

    let mut stmt = conn
        .prepare_cached(
            "SELECT * FROM tags 
            JOIN post_tags ON post_tags.tag = tags.id
            WHERE post_tags.post = ?",
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut rows = stmt.query([query.post]).unwrap();

    let mut data = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        let id = row.get_unwrap(0);
        let name = row.get_unwrap::<_, String>(1);

        data.push(Tag { id, name });
    }

    Ok(APIResponse { data })
}

async fn get_tags_api(State(state): State<AppState>) -> Result<APIResponse<Vec<Tag>>, StatusCode> {
    let conn = state.conn();

    let mut stmt = conn
        .prepare_cached("SELECT * FROM tags;")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut rows = stmt.query([]).unwrap();

    let mut data = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        let id = row.get_unwrap(0);
        let name = row.get_unwrap::<_, String>(1);

        data.push(Tag { id, name });
    }

    Ok(APIResponse { data })
}

fn get_file_meta(
    state: &AppState,
    conn: &rusqlite::Connection,
    id: FileMetaId,
) -> Result<Value, rusqlite::Error> {
    let mut stmt = conn.prepare_cached("SELECT * FROM file_metas WHERE id = ?")?;

    match stmt
        .query_row([id.raw()], |row| {
            let id = FileMetaId(row.get_unwrap(0));
            let filename = row.get_unwrap::<_, String>(1);
            let author = AuthorId(row.get_unwrap(2));
            let post = PostId(row.get_unwrap(3));
            let mime = row.get_unwrap::<_, String>(4);
            let extra: Value = serde_json::from_str(&row.get_unwrap::<_, String>(5)).unwrap();
            let url = format!(
                "{}/{}/{}/{}",
                state.static_url(&mime),
                author,
                post,
                filename
            );

            Ok(json!({
                "id":id,
                "url":url,
                "mime":mime,
                "extra":extra,
            }))
        })
        .optional()?
    {
        Some(data) => Ok(data),
        // return
        None => Ok(json!({
            "id": -1,
            "url":"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='1em' height='1em' viewBox='0 0 24 24'%3E%3Cpath fill='none' stroke='currentColor' stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M3 7v4a1 1 0 0 0 1 1h3m0-5v10m3-9v8a1 1 0 0 0 1 1h2a1 1 0 0 0 1-1V8a1 1 0 0 0-1-1h-2a1 1 0 0 0-1 1m7-1v4a1 1 0 0 0 1 1h3m0-5v10'/%3E%3C/svg%3E",
            "mime":"image/svg+xml",
            "extra":{},
        })),
    }
}
