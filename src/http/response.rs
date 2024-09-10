use std::fmt::Display;
use std::fs;
use std::io;
use walkdir::WalkDir;
use infer;
use url_escape::encode_component;

use super::request::HttpRequest;
use super::request::Version;

#[derive(Debug)]
pub struct HttpResponse {
    pub version: Version,
    pub status: ResponseStatus,
    pub content_length: usize,
    pub accept_ranges: AcceptRanges,
    pub response_body: String,
    pub current_path: String,
}

impl HttpResponse {
    pub fn new(request: &HttpRequest) -> io::Result<HttpResponse> {
        let version = Version::V1_1;
        let mut status = ResponseStatus::NotFound;
        let mut content_length = 0;
        let mut accept_ranges = AcceptRanges::None;
        let current_path = request.resource.path.clone();
        let mut response_body = String::new();

        let server_root_path = std::env::current_dir()?;
        let resource_path = server_root_path.join(&request.resource.path);

        // Prevent backtracking by checking canonicalized paths
        let root_path_len = server_root_path.canonicalize()?.components().count();
        let resource_path_len = resource_path.canonicalize()?.components().count();

        if root_path_len > resource_path_len {
            // Backtracking detected
            response_body.push_str("Backtracking is not allowed.");
            status = ResponseStatus::NotFound;
        } else if resource_path.exists() {
            if resource_path.is_file() {
                // Serve file content
                let content = fs::read(&resource_path)?;
                content_length = content.len();
                status = ResponseStatus::OK;
                accept_ranges = AcceptRanges::Bytes;

                // Get MIME type using infer crate, default to "application/octet-stream"
                let mime_type = infer::get_from_path(&resource_path)?
                    .map(|t| t.mime_type())
                    .unwrap_or("application/octet-stream");

                let headers = format!(
                    "{} {}\n{}\ncontent-length: {}\ncontent-type: {}\r\n\r\n",
                    version, status, accept_ranges, content_length, mime_type
                );
                response_body.push_str(&headers);
                response_body.push_str(&String::from_utf8_lossy(&content));
            } else if resource_path.is_dir() {
                // Serve directory content as HTML
                let mut html_body = String::from(
                    r#"
                    <!DOCTYPE html>
                    <html>
                    <head>
                        <meta charset="utf-8">
                        <title>Directory Listing</title>
                    </head>
                    <body>"#,
                );

                let header = format!("<h1>Currently in {}</h1>", current_path);
                html_body.push_str(&header);

                for entry in WalkDir::new(&resource_path).min_depth(1).max_depth(1) {
                    let entry = entry?;
                    let path = entry.path();
                    let file_name = path.file_name().unwrap().to_string_lossy();
                    let encoded_name = encode_component(&file_name);
                    let file_link = format!(r#"<a href="/{}">{}</a><br>"#, encoded_name, file_name);
                    html_body.push_str(&file_link);
                }

                html_body.push_str("</body></html>");
                content_length = html_body.len();
                response_body.push_str(&html_body);
                status = ResponseStatus::OK;
            }
        } else {
            // 404 Page Not Found
            let not_found_page = "<html><body><h1>404 NOT FOUND</h1></body></html>";
            content_length = not_found_page.len();
            response_body.push_str(not_found_page);
        }

        Ok(HttpResponse {
            version,
            status,
            content_length,
            accept_ranges,
            response_body,
            current_path,
        })
    }
}

#[derive(Debug)]
pub enum ResponseStatus {
    OK = 200,
    NotFound = 404,
}

impl Display for ResponseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            ResponseStatus::OK => "200 OK",
            ResponseStatus::NotFound => "404 Not Found",
        };
        write!(f, "{}", msg)
    }
}

#[derive(Debug)]
pub enum AcceptRanges {
    Bytes,
    None,
}

impl Display for AcceptRanges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            AcceptRanges::Bytes => "accept-ranges: bytes",
            AcceptRanges::None => "accept-ranges: none",
        };
        write!(f, "{}", msg)
    }
}
