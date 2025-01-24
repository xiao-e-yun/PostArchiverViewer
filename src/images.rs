use std::{collections::HashMap, fs, io::Cursor, path::PathBuf};

use axum::{
    body::Bytes,
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, Uri},
    response::IntoResponse,
    Router,
};
use axum_response_cache::CacheLayer;
use fast_image_resize::{images::Image, IntoImageView, ResizeOptions, Resizer};
use image::{
    codecs::{jpeg::JpegEncoder, png::PngEncoder, webp::WebPEncoder},
    ColorType, ImageEncoder, ImageFormat,
};
use mime_guess::MimeGuess;

pub fn get_images_router<P: AsRef<std::path::Path>>(path: P) -> Router {
    let cache = CacheLayer::with_lifespan(86400);

    let path = path.as_ref().to_path_buf().canonicalize().unwrap();
    Router::new()
        .fallback(provide_images)
        .with_state(path)
        .layer(cache)
}

async fn provide_images(
    State(root): State<PathBuf>,
    Query(query): Query<HashMap<String, String>>,
    uri: Uri,
) -> Result<(HeaderMap, Bytes), StatusCode> {
    let path = safe_path::scoped_join(root, uri.path()).unwrap();

    if !path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    let Some(mime) = match query.get("output") {
        Some(ext) => MimeGuess::from_ext(ext),
        None => MimeGuess::from_path(&path),
    }
    .first() else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let Some(format) = ImageFormat::from_mime_type(&mime) else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        HeaderValue::from_str(mime.as_ref()).unwrap(),
    );
    headers.insert(
        "Cache-Control",
        HeaderValue::from_static("public, max-age=31536000"),
    );

    let dpr: u32 = query.get("dpr").and_then(|s| s.parse().ok()).unwrap_or(1);

    let dst_width: Option<u32> = query.get("w").and_then(|s| s.parse().ok());
    let dst_height: Option<u32> = query.get("h").and_then(|s| s.parse().ok());
    if dst_width.is_none() && dst_height.is_none() {
        let bytes = fs::read(path)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .into();
        return Ok((headers, bytes));
    }

    let image = image::open(&path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let src_width = image.width();
    let src_height = image.height();
    let aspect_ratio = src_width as f32 / src_height as f32;

    //Weserv-like image resizing
    let (dst_width, dst_height) = match (dst_width, dst_height) {
        (Some(dst_width), Some(dst_height)) => (dst_width * dpr, dst_height * dpr),
        (Some(dst_width), None) => {
            let dst_height = (dst_width as f32 / aspect_ratio).round() as u32;
            (dst_width * dpr, dst_height * dpr)
        }
        (None, Some(dst_height)) => {
            let dst_width = (dst_height as f32 * aspect_ratio).round() as u32;
            (dst_width * dpr, dst_height * dpr)
        }
        (None, None) => unreachable!(),
    };

    let mut dst_image = Image::new(dst_width, dst_height, image.pixel_type().unwrap());

    let mut resizer = Resizer::new();

    let options = ResizeOptions::new()
        .resize_alg(fast_image_resize::ResizeAlg::Nearest)
        .fit_into_destination(Some((0.5, 0.5)));
    resizer
        .resize(&image, &mut dst_image, Some(&options))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut buffer = vec![];

    match format {
        ImageFormat::WebP => write_image(
            WebPEncoder::new_lossless(&mut buffer),
            dst_image,
            dst_width,
            dst_height,
            image.color(),
        ),
        ImageFormat::Png => write_image(
            PngEncoder::new(&mut buffer),
            dst_image,
            dst_width,
            dst_height,
            image.color(),
        ),
        ImageFormat::Jpeg => write_image(
            JpegEncoder::new(&mut buffer),
            dst_image,
            dst_width,
            dst_height,
            image.color(),
        ),
        _ => todo!(),
    }?;

    fn write_image(
        encoder: impl ImageEncoder,
        dst_image: Image<'_>,
        dst_width: u32,
        dst_height: u32,
        color: ColorType,
    ) -> Result<(), StatusCode> {
        encoder
            .write_image(dst_image.buffer(), dst_width, dst_height, color.into())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    Ok((headers, buffer.into()))
}
