use std::{
    io::{self, Read, Write}, // Input/Output operations
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream}, // Networking modules for IPv4, sockets, and streams
    path::Path, // For handling file paths
    fs, // For file system operations
    env, // For handling environment variables
};

use simple_http::http::{request, response}; // Importing the request and response modules from the custom `simple_http::http`

// Function to create a socket address (IPv4 localhost at port 5500)
fn create_socket() -> SocketAddr {
    SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::LOCALHOST), 5500)
}

// Function to handle individual client connections
fn handle_client(stream: &mut TcpStream) -> io::Result<()> {
    let mut buffer = [0; 1024]; // Buffer to store incoming client data
    stream.read(&mut buffer)?; // Read data from the client into the buffer

    let buf_str = String::from_utf8_lossy(&buffer); // Convert the buffer into a UTF-8 string
    let request = request::HttpRequest::new(&buf_str)?; // Create a new HttpRequest object from the string

    let response = request.response()?; // Generate the appropriate HttpResponse based on the request

    println!("{:?}", &response); // Print the response for debugging purposes

    // Create the HTTP response headers (including content length and type)
    let headers = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n",
        response.content_length, response.content_type
    );

    // Write the headers and response body to the stream, sending the response to the client
    stream.write(headers.as_bytes())?;
    stream.write(&response.response_body)?;
    stream.flush()?; // Ensure all data is written to the client

    Ok(())
}

// Function to listen for incoming connections and handle each client
fn serve(socket: SocketAddr) -> io::Result<()> {
    let listener = TcpListener::bind(socket)?; // Bind the socket to listen for incoming connections
    let mut counter = 0; // Counter to track the number of client connections

    // Loop through each incoming connection
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                // Spawn a new thread to handle the client connection
                match std::thread::spawn(move || handle_client(&mut stream)).join() {
                    Ok(_) => {
                        counter += 1; // Increment the counter for each successful connection
                        println!("connected stream... {}", counter); // Print connection number
                    }
                    Err(_) => continue, // If the thread fails, continue to the next client
                };
            }
            Err(e) => {
                // Print any errors that occur while accepting a client connection
                eprintln!("Failed to accept a client: {}", e);
            }
        }
    }
    Ok(())
}

// Main function to start the server
fn main() -> io::Result<()> {
    let socket = create_socket(); // Create a socket on localhost:5500
    serve(socket)?; // Start the server and listen for incoming connections
    Ok(())
}
