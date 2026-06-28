//! The embedded SolidJS single-page app. In release builds the `web-ui/dist`
//! tree is baked into the binary; in debug builds rust-embed reads it from
//! disk so frontend rebuilds show up without recompiling Rust.

use actix_web::{HttpRequest, HttpResponse};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "web-ui/dist"]
struct Assets;

fn serve_path(path: &str) -> HttpResponse {
    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            HttpResponse::Ok()
                .content_type(mime.as_ref())
                .body(content.data.into_owned())
        }
        None => HttpResponse::NotFound().finish(),
    }
}

/// Default service: serve the requested asset, falling back to `index.html`
/// for unknown paths so client-side routing works.
pub async fn spa(req: HttpRequest) -> HttpResponse {
    let path = req.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    if Assets::get(path).is_some() {
        return serve_path(path);
    }
    // SPA fallback.
    serve_path("index.html")
}
