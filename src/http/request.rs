use std::{collections::HashMap, fmt::Display, io, str::FromStr}; // Imports needed for handling HashMap, formatting, I/O, and string parsing
use super::response::HttpResponse; // Import HttpResponse from the response module

// The HttpRequest struct stores information about an HTTP request
#[derive(Debug)]
pub struct HttpRequest {
    method: Method,         // HTTP method (GET, POST, etc.)
    pub resource: Resource, // Requested resource (e.g., file path)
    version: Version,       // HTTP version (1.1, 2.0)
    headers: HttpHeader,    // HTTP headers (key-value pairs)
    pub request_body: String, // Body of the HTTP request (for POST, etc.)
}

impl HttpRequest {
    // Method to generate an HTTP response for the current request
    pub fn response(&self) -> io::Result<HttpResponse> {
        HttpResponse::new(self)
    }

    // Constructs a new HttpRequest from the raw request string
    pub fn new(request: &str) -> io::Result<HttpRequest> {
        let method = Method::new(request); // Extract method (GET, POST, etc.)
        let resource = Resource::new(request).unwrap_or_else(|| Resource {
            path: "".to_string(),
        }); // Extract requested resource path
        let version = Version::new(request)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.msg))?; // Extract version or return an error
        let headers = HttpHeader::new(request).unwrap_or(HttpHeader {
            headers: HashMap::new(),
        }); // Extract headers
        let request_body = request.split_once("\r\n\r\n").map_or(String::new(), |(_, body)| body.to_string()); // Extract body of the request

        Ok(HttpRequest {
            method,
            resource,
            version,
            headers,
            request_body,
        })
    }
}

// Represents the headers of the HTTP request as a HashMap of key-value pairs
#[derive(Debug)]
struct HttpHeader {
    headers: HashMap<String, String>,
}

impl HttpHeader {
    // Parses headers from the raw request string and returns an HttpHeader struct
    pub fn new(request: &str) -> Option<HttpHeader> {
        let mut httpheader = HttpHeader {
            headers: HashMap::new(),
        };
        let (_, header_str) = request.split_once("\r\n")?; // Extract headers portion from the request
        for line in header_str.split_terminator("\r\n") {
            if line.is_empty() {
                break;
            }
            let (header, value) = line.split_once(":")?; // Split header lines into key-value pairs
            httpheader.headers.insert(header.trim().to_string(), value.trim().to_string());
        }
        Some(httpheader)
    }
}

// Enum representing the HTTP version (1.1, 2.0)
#[derive(Debug)]
pub enum Version {
    V1_1,
    V2_0,
}

impl Display for Version {
    // Implements the Display trait for formatting the version as a string
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Version::V1_1 => "HTTP/1.1",
            Version::V2_0 => "HTTP/2",
        };
        write!(f, "{}", msg)
    }
}

// Error struct for handling invalid version parsing
pub struct VersionError {
    msg: String,
}

impl Display for VersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Version {
    // Creates a new Version object from the raw request string
    pub fn new(request: &str) -> Result<Self, VersionError> {
        Version::from_str(request)
    }
}

impl FromStr for Version {
    type Err = VersionError;

    // Parses the version from the request string
    fn from_str(request: &str) -> Result<Self, Self::Err> {
        let request_split = request.split_once("\r\n");
        if let Some((method_line, _rest)) = request_split {
            let splits = method_line.split_ascii_whitespace(); // Split the method line (e.g., GET / HTTP/1.1)
            for split in splits {
                if split == "HTTP/1.1" {
                    return Ok(Version::V1_1);
                } else if split == "HTTP/2" || split == "HTTP/2.0" {
                    return Ok(Version::V2_0);
                };
            }
        }
        let invalid = format!("Unknown protocol version in {}", request);
        let version_error = VersionError { msg: invalid };
        Err(version_error)
    }
}

// Enum representing the HTTP method (GET, POST, or Uninitialized)
#[derive(Debug)]
enum Method {
    Get,
    Post,
    Uninitialized,
}

impl Method {
    // Parses the method (GET, POST, etc.) from the raw request string
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
}

// Struct representing the requested resource (e.g., a file path)
#[derive(Debug)]
pub struct Resource {
    pub path: String,
}

impl Resource {
    // Parses the resource path from the request string
    pub fn new(request: &str) -> Option<Resource> {
        request.split_once("\r\n").and_then(|(method_line, _)| {
            method_line.split_once(' ').and_then(|(_, rest)| {
                rest.split_once(' ').map(|(resource, _)| {
                    Resource {
                        path: resource.trim_start_matches('/').to_string(),
                    }
                })
            })
        })
    }
}
