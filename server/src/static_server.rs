use axum::body::{self, Full};
use axum::handler::Handler;
use axum::http::header::HeaderName;
use axum::http::{header, HeaderValue, Response, StatusCode, Uri};
use axum::response::IntoResponse;
use axum::routing::{get, MethodRouter};
use bytes::Bytes;
use http_body::combinators::UnsyncBoxBody;
use include_dir::include_dir;
use once_cell::sync::Lazy;
use pulldown_cmark::{html, Parser};
use std::path::Path;
use tera::{Context, Tera};
use tower_http::set_header::SetResponseHeaderLayer;

pub const CDN_CACHE_CONTROL_HEADER: HeaderName = HeaderName::from_static("cdn-cache-control");

static STATIC_DIR: include_dir::Dir = include_dir!("public");
static TERA: Lazy<Tera> = Lazy::new(|| match Tera::new("templates/**/*") {
    Ok(t) => t,
    Err(e) => {
        println!("Parsing error(s): {}", e);
        ::std::process::exit(1);
    }
});

fn mime_type_from_path<P>(path: P) -> mime_guess::Mime
where
    P: AsRef<Path>,
{
    mime_guess::from_path(path).first_or_text_plain()
}

fn bytes_to_response(
    bytes: http_body::Full<Bytes>,
    mime_type: mime_guess::Mime,
) -> Response<UnsyncBoxBody<Bytes, axum::Error>> {
    Response::builder()
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_str(mime_type.as_ref()).unwrap(),
        )
        .status(StatusCode::OK)
        .body(body::boxed(bytes))
        .unwrap()
}

fn file_to_response(
    file: &'static include_dir::File,
) -> Response<UnsyncBoxBody<Bytes, axum::Error>> {
    bytes_to_response(
        Full::from(file.contents()),
        mime_type_from_path(file.path()),
    )
}

fn convert_md_to_html(content: &str) -> Vec<u8> {
    let parser = Parser::new(content);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    let mut context = Context::new();
    context.insert("html", &html_output);

    let mut buf = Vec::new();
    TERA.render_to("base.html", &context, &mut buf).unwrap();

    buf
}

async fn static_path(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let has_extension = path.split_once('.').is_some();
    let not_found_response = StatusCode::NOT_FOUND.into_response();

    if has_extension {
        let path = if path.is_empty() { "index.html" } else { path };
        return STATIC_DIR
            .get_file(path)
            .map_or(not_found_response, file_to_response);
    }

    if path.is_empty() {
        return STATIC_DIR
            .get_file("index.html")
            .map_or(StatusCode::NOT_FOUND.into_response(), file_to_response);
    }

    let md = format!("{}.md", path);
    STATIC_DIR.get_file(md).map_or(not_found_response, |file| {
        let bytes = convert_md_to_html(file.contents_utf8().unwrap_or_default());
        bytes_to_response(Full::from(bytes), mime_guess::mime::TEXT_HTML_UTF_8)
    })
}

pub fn file_service(browser_max_age: HeaderValue, cdn_max_age: HeaderValue) -> MethodRouter {
    let static_handler = static_path
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CACHE_CONTROL,
            browser_max_age,
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            CDN_CACHE_CONTROL_HEADER,
            cdn_max_age,
        ));
    get(static_handler)
}
