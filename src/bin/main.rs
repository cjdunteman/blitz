use std::fs;
use std::thread;
use std::time::Duration;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use blitz::ThreadPool;

fn main() {
    // listen for TCP connections
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    // thread pool
    let pool = ThreadPool::new(2);

    // process each connection and produce a series of streams to handle
    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

// read data from TCP stream and print it
fn handle_connection(mut stream: TcpStream) {
    // buffer is 1024 bytes - buffer mngmt would be more complicated
    // for handling requests of an arbitrary size
    let mut buffer = [0; 1024];

    // take 8 bits and turn into String
    stream.read(&mut buffer).unwrap();

    // hardcode data corresponding to the / request into get var
    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    // check if buffer starts with the bytes in get
    // if it does, we've receive a well-formed request to /
    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "index.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "index.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };
    
    let contents = fs::read_to_string(filename).unwrap();

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    // convert to bytes
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
