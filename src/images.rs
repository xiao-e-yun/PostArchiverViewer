use std::{collections::HashMap, fs};

use axum::{
    body::Bytes,
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, Uri},
    Router,
};
use axum_response_cache::CacheLayer;
use cached::TimedSizedCache;
use fast_image_resize::{
    images::Image, FilterType, IntoImageView, ResizeAlg, ResizeOptions, Resizer,
};
use image::{
    codecs::{jpeg::JpegEncoder, png::PngEncoder, webp::WebPEncoder},
    ColorType, ImageEncoder, ImageFormat,
};
use mime_guess::MimeGuess;
use tracing::info;

use crate::config::Config;

pub fn get_images_router(config: &Config) -> Router<()> {
    if config.public.images_url.is_some() {
        info!("Images URL is set, disabling images router");
        return Router::new().fallback(|| async { StatusCode::FORBIDDEN });
    }

    let mut router = Router::new()
        .fallback(provide_images)
        .with_state(config.clone());

    let size = config.resize.cache_size;
    if size != 0 {
        let cache =
            TimedSizedCache::with_size_and_lifespan_and_refresh(size, 30 * 24 * 60 * 60, true);
        let cache = CacheLayer::with(cache);
        router = router.layer(cache);
    }

    router
}

async fn provide_images(
    State(config): State<Config>,
    Query(query): Query<HashMap<String, String>>,
    uri: Uri,
) -> Result<(HeaderMap, Bytes), StatusCode> {
    let root = &config.path;
    let path = path_clean::clean(urlencoding::decode(uri.path()).unwrap().as_ref());

    let Ok(path) = path.strip_prefix("/") else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let path = root.join(path);

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

    let algorithm = if cfg!(debug_assertions) {
        ResizeAlg::Nearest
    } else {
        let filter_type = match config.resize.filter_type.as_str() {
            "lanczos3" => FilterType::Lanczos3,
            "gaussian" => FilterType::Gaussian,
            "catmull-rom" => FilterType::CatmullRom,
            "hamming" => FilterType::Hamming,
            "mitchell" => FilterType::Mitchell,
            "bilinear" => FilterType::Bilinear,
            "box" => FilterType::Box,
            _ => FilterType::Lanczos3,
        };

        match config.resize.algorithm.as_str() {
            "super-sampling8x" => ResizeAlg::SuperSampling(filter_type, 8),
            "super-sampling4x" => ResizeAlg::SuperSampling(filter_type, 4),
            "super-sampling2x" => ResizeAlg::SuperSampling(filter_type, 2),
            "interpolation" => ResizeAlg::Interpolation(filter_type),
            "convolution" => ResizeAlg::Convolution(filter_type),
            "nearest" => ResizeAlg::Nearest,
            _ => ResizeAlg::Interpolation(filter_type),
        }
    };
    let options = ResizeOptions::new()
        .resize_alg(algorithm)
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
