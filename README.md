# Simple HTTP Server

A simple HTTP server implemented in Rust. This project handles HTTP requests and serves various types of files including text, MP3 songs, MP4 videos, and Chinese text.

## Features

- Serve static files (text, MP3, MP4, etc.)
- Serve directories with HTML listing
- Handle HTTP GET requests
- Response with appropriate MIME types
- Handle Accept-Ranges for byte-range requests

## Project Structure

- `src/main.rs`: Entry point of the application
- `src/lib.rs`: Library file
- `src/http/`: Contains HTTP request and response handling
  - `mod.rs`: Module definitions
  - `request.rs`: HTTP request handling
  - `response.rs`: HTTP response handling

## Installation

Make sure you have [Rust](https://www.rust-lang.org/learn/get-started) installed on your machine. Clone the repository and build the project using Cargo.

```sh
git clone https://github.com/yourusername/your-repo.git
cd your-repo
cargo build
