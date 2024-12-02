//! Webicon: Fetch facvicons "fast".
//!
//! Copyright (C) 2024 Ari Prakash
//!
//! This program is free software:
//! you can redistribute it and/or modify it under the terms of the GNU Affero
//! General Public License as published by the Free Software Foundation, either
//! version 3 of the License, or (at your option) any later version.
//!
//! This program is
//! distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without
//! even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
//! See the GNU Affero General Public License for more details.
//!
//! You should have
//! received a copy of the GNU Affero General Public License along with this program.
//! If not, see <https://www.gnu.org/licenses/>.

#![warn(clippy::pedantic)]
#![allow(clippy::wildcard_imports)]

use std::{error::Error, io::Cursor};

use axum::{
    extract::Path,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use image::{codecs::png::PngEncoder, ImageReader};
use reqwest::{header::USER_AGENT, Client};
use scraper::{Html, Selector};
use worker::*;

const HOSTNAME: &str = "webicon.ariscript.org";
const ALLOWED_ORIGINS: &str = "*.ariscript.org";
const CACHE_CONTROL: &str =
    "public, s-maxage=604800, stale-while-revalidate=86400";

pub fn router() -> Router {
    Router::new()
        .route("/favicon.ico", get(favicon))
        .route("/*url", get(root))
}

#[cfg(target_arch = "wasm32")]
#[event(fetch)]
#[allow(clippy::no_effect_underscore_binding)]
async fn fetch(
    req: HttpRequest,
    _env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    use tower_service::Service;

    console_error_panic_hook::set_once();
    Ok(router().call(req).await?)
}

pub async fn root(Path(url): Path<String>) -> impl IntoResponse {
    let mut headers = HeaderMap::with_capacity(1);
    headers.append(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36".parse().expect("this is a valid header"));

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .expect("client should build");

    match icon(&url, &client).await {
        Ok(img) => (
            StatusCode::OK,
            [
                ("Content-Type", "image/png"),
                ("Access-Control-Allow-Origin", ALLOWED_ORIGINS),
                ("Cache-Control", CACHE_CONTROL),
            ],
            img,
        ),
        Err(e) => (
            StatusCode::NOT_FOUND,
            [
                ("Content-Type", "text/plain"),
                ("Access-Control-Allow-Origin", "*"),
                ("Cache-Control", CACHE_CONTROL),
            ],
            format!("{e:?}").into(),
        ),
    }
}

pub async fn favicon() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "").into_response()
}

// using `#[worker::send]` is fine here since the worker environment is always
// single-threaded, but axum requires `Send` futures (the futures from `Fetch`
// are not `Send`). This will never cause any problems with thread-safety since
// there _are no other threads_, unless Cloudflare changes this, which should
// surely be an opt-in feature.
#[worker::send]
async fn icon(
    url: &str,
    client: &Client,
) -> std::result::Result<Vec<u8>, Box<dyn Error>> {
    let url = Url::parse(url)?;

    if url.host_str().is_some_and(|h| h == HOSTNAME) {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "circular request",
        )) as Box<dyn Error>);
    }

    let html = client.get(url.as_str()).send().await?.text().await?;

    let doc = Html::parse_document(&html);
    let selectors = [
        "link[rel=\"apple-touch-icon\"]",
        "link[rel=\"apple-touch-icon-precomposed\"]",
        "link[rel~=\"icon\"]",
        "link[rel=\"mask-icon\"]",
    ]
    .map(|s| Selector::parse(s).expect("this is a valid CSS selector"));

    let icon_urls: Vec<_> = selectors
        .iter()
        .flat_map(|r| doc.select(r))
        .filter_map(|icon| url.join(icon.attr("href")?).ok())
        .collect();

    for icon_url in icon_urls {
        if let Ok(i) = url_to_icon(icon_url, client).await {
            return Ok(i);
        }
    }

    url_to_icon(url.join("/favicon.ico")?, client).await
}

async fn url_to_icon(
    url: Url,
    client: &Client,
) -> std::result::Result<Vec<u8>, Box<dyn Error>> {
    let buf: Vec<u8> =
        client.get(url.as_str()).send().await?.bytes().await?.into();

    let mut out = Vec::with_capacity(buf.capacity());
    let encoder = PngEncoder::new(&mut out);

    let mut img = ImageReader::new(Cursor::new(&buf))
        .with_guessed_format()?
        .decode()?;

    // if the favicon is too small already, I'll leave it to the user to figure it out
    if img.width() > 64 || img.height() > 64 {
        img = img.thumbnail(64, 64);
    }

    img.write_with_encoder(encoder)?;
    Ok(out)
}
