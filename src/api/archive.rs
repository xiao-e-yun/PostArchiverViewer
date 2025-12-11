use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::Response,
    Json,
};
use bytes::Bytes;
use post_archiver::FileMetaId;
use serde::{Deserialize, Serialize};
use std::io::Read;
use ts_rs::TS;
use zip::ZipArchive;

use crate::api::AppState;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct ArchiveEntry {
    pub name: String,
    pub size: u64,
    pub is_image: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct ArchiveListResponse {
    pub entries: Vec<ArchiveEntry>,
}

#[derive(Debug, Deserialize)]
pub struct ExtractQuery {
    pub file: String,
}

/// Get file path from FileMeta ID
fn get_file_path(state: &AppState, file_id: FileMetaId) -> Result<std::path::PathBuf, StatusCode> {
    let manager = state.manager();
    let conn = manager.conn();

    let mut stmt = conn
        .prepare_cached("SELECT filename, post FROM file_metas WHERE id = ?")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (filename, post): (String, u32) = stmt
        .query_row([file_id], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let folder = post / 2048;
    let subfolder = post % 2048;
    
    Ok(state.config.path.join(format!("{}/{}/{}", folder, subfolder, filename)))
}

/// Check if a file is an image based on its name
fn is_image_file(name: &str) -> bool {
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "svg")
}

/// List files in a compressed archive
pub async fn list_archive_handler(
    Path(file_id): Path<FileMetaId>,
    State(state): State<AppState>,
) -> Result<Json<ArchiveListResponse>, StatusCode> {
    let file_path = get_file_path(&state, file_id)?;
    
    let file = std::fs::File::open(&file_path).map_err(|_| StatusCode::NOT_FOUND)?;
    let mut archive = ZipArchive::new(file).map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut entries = Vec::new();
    for i in 0..archive.len() {
        let file = archive.by_index(i).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        // Skip directories
        if file.is_dir() {
            continue;
        }

        let name = file.name().to_string();
        let size = file.size();
        let is_image = is_image_file(&name);

        entries.push(ArchiveEntry {
            name,
            size,
            is_image,
        });
    }

    Ok(Json(ArchiveListResponse { entries }))
}

/// Extract and serve a file from a compressed archive
pub async fn extract_file_handler(
    Path(file_id): Path<FileMetaId>,
    Query(query): Query<ExtractQuery>,
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    // Validate the file path to prevent path traversal attacks
    let path = std::path::Path::new(&query.file);
    for component in path.components() {
        match component {
            std::path::Component::Normal(_) => continue,
            // Reject any path with parent directory references or other unsafe components
            _ => return Err(StatusCode::BAD_REQUEST),
        }
    }

    let file_path = get_file_path(&state, file_id)?;
    
    let file = std::fs::File::open(&file_path).map_err(|_| StatusCode::NOT_FOUND)?;
    let mut archive = ZipArchive::new(file).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Find the requested file in the archive
    let mut zip_file = archive
        .by_name(&query.file)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Read the file content
    let mut buffer = Vec::new();
    zip_file
        .read_to_end(&mut buffer)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Guess the MIME type
    let mime = mime_guess::from_path(&query.file)
        .first_or_octet_stream()
        .to_string();

    // Build response
    let body = Body::from(Bytes::from(buffer));
    
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime)
        .body(body)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[cfg(test)]
mod tests {
    use ts_rs::TS;

    #[test]
    fn export_bindings_archiveentry() {
        super::ArchiveEntry::export().unwrap();
    }

    #[test]
    fn export_bindings_archivelistresponse() {
        super::ArchiveListResponse::export().unwrap();
    }
}
