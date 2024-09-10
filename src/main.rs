use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use simple_file_server::http::request::HttpRequest;
use simple_file_server::http::response::HttpResponse;

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024]; // Buffer to read request data
    let bytes_read = match stream.read(&mut buffer) {
        Ok(size) => size,
        Err(e) => {
            eprintln!("Failed to read from stream: {}", e);
            return;
        }
    };

    if bytes_read == 0 {
        return; // Exit if no data read
    }

    // Convert request to string
    let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);

    // Parse the HTTP request
    match HttpRequest::new(&request_str) {
        Ok(request) => {
            // Generate the HTTP response based on the request
            match HttpResponse::new(&request) {
                Ok(response) => {
                    // Write the response back to the client
                    if let Err(e) = stream.write_all(response.response_body.as_bytes()) {
                        eprintln!("Failed to send response: {}", e);
                    }
                }
                Err(e) => eprintln!("Failed to generate response: {}", e),
            }
        }
        Err(e) => eprintln!("Failed to parse request: {}", e),
    }

    // Flush the stream to ensure the response is sent
    if let Err(e) = stream.flush() {
        eprintln!("Failed to flush stream: {}", e);
    }
}

fn main() -> std::io::Result<()> {
    // Bind the server to the local IP address and port 7878
    let listener = TcpListener::bind("127.0.0.1:7878")?;

    println!("Server running on 127.0.0.1:7878");

    // Loop to handle incoming connections
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Handle each client connection in a separate function
                handle_client(stream);
            }
            Err(e) => eprintln!("Failed to establish connection: {}", e),
        }
    }

    Ok(())
}
