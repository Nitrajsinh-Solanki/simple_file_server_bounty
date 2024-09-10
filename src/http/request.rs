use std::collections::HashMap;
use std::io;
use std::str::FromStr;
use std::fmt::{self, Display};

use super::response::HttpResponse;

#[derive(Debug)]
pub struct HttpRequest {
    pub method: Method,
    pub resource: Resource,
    pub version: Version,
    pub headers: HttpHeader,
    pub request_body: String,
}

impl HttpRequest {
    pub fn response(&self) -> io::Result<HttpResponse> {
        HttpResponse::new(self)
    }

    pub fn new(request: &str) -> io::Result<HttpRequest> {
        let method: Method = Method::new(request);
        let resource: Resource = Resource::new(request).unwrap_or_else(|| Resource {
            path: "".to_string(),
        });
        let version: Version = Version::new(request)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.msg))?;
        let headers: HttpHeader = HttpHeader::new(request).unwrap_or_else(|| HttpHeader {
            headers: HashMap::new(),
        });

        let request_body = if let Some((_header, body)) = request.split_once("\r\n\r\n") {
            body.to_string()
        } else {
            String::new()
        };

        Ok(HttpRequest {
            method,
            resource,
            version,
            headers,
            request_body,
        })
    }
}

#[derive(Debug)]
pub struct HttpHeader {
    headers: HashMap<String, String>,
}

impl HttpHeader {
    pub fn new(request: &str) -> Option<HttpHeader> {
        let mut httpheader = HttpHeader {
            headers: HashMap::new(),
        };
        let (_, header_str) = request.split_once("\r\n\r\n")?; // Split header and body
        for line in header_str.split_terminator("\r\n") {
            if line.is_empty() {
                break;
            }
            let (header, value) = line.split_once(":")?;
            httpheader
                .headers
                .insert(header.trim().to_string(), value.trim().to_string());
        }
        Some(httpheader)
    }
}

#[derive(Debug)]
pub enum Version {
    V1_0,
    V1_1,
    V2_0, // Added this variant
}

impl Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Version::V1_0 => write!(f, "HTTP/1.0"),
            Version::V1_1 => write!(f, "HTTP/1.1"),
            Version::V2_0 => write!(f, "HTTP/2.0"), // Added this
        }
    }
}

impl Version {
    pub fn new(request: &str) -> Result<Self, VersionError> {
        Version::from_str(request)
    }
}

impl FromStr for Version {
    type Err = VersionError;

    fn from_str(request: &str) -> Result<Self, Self::Err> {
        if let Some((method_line, _)) = request.split_once("\r\n") {
            for split in method_line.split_ascii_whitespace() {
                if split == "HTTP/1.0" {
                    return Ok(Version::V1_0);
                } else if split == "HTTP/1.1" {
                    return Ok(Version::V1_1);
                } else if split == "HTTP/2" || split == "HTTP/2.0" {
                    return Ok(Version::V2_0);
                }
            }
        }

        let invalid = format!("Unknown protocol version in {}", request);
        let version_error = VersionError { msg: invalid };
        Err(version_error)
    }
}

pub struct VersionError {
    pub msg: String,
}

#[derive(Debug)]
pub enum Method {
    Get,
    Post,
    Uninitialized,
}

impl Method {
    pub fn new(request: &str) -> Method {
        if let Some((method_line, _)) = request.split_once("\r\n") {
            if let Some((method, _)) = method_line.split_once(' ') {
                return match method {
                    "GET" => Method::Get,
                    "POST" => Method::Post,
                    _ => Method::Uninitialized,
                };
            }
        }
        Method::Uninitialized
    }

    pub fn identify(s: &str) -> Method {
        match s {
            "GET" => Method::Get,
            "POST" => Method::Post,
            _ => Method::Uninitialized,
        }
    }
}

#[derive(Debug)]
pub struct Resource {
    pub path: String,
}

impl Resource {
    pub fn new(request: &str) -> Option<Resource> {
        if let Some((method_line, _)) = request.split_once("\r\n") {
            if let Some((_, rest)) = method_line.split_once(' ') {
                let (resource, _) = rest.split_once(' ')?;
                let resource = resource.trim_start_matches('/');
                return Some(Resource {
                    path: resource.to_string(),
                });
            }
        }
        None
    }
}
