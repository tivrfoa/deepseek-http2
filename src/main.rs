use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

/*
Deepseek:

Conclusion

This is a very basic starting point for an HTTP/2 server in Rust. A fully functional HTTP/2 server would require handling many more details, such as flow control, stream prioritization, header compression (HPACK), and more. Implementing all of this from scratch is a significant undertaking, which is why most developers use existing libraries like hyper or h2 for HTTP/2 in Rust.

If you want to build a production-ready HTTP/2 server, I highly recommend using these libraries, as they handle all the complexities of the protocol for you.
*/

struct Http2Frame {
    length: u32,
    type_: u8,
    flags: u8,
    stream_id: u32,
    payload: Vec<u8>,
}

impl Http2Frame {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 9 {
            return None;
        }

        let length = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], 0]);
        let type_ = bytes[3];
        let flags = bytes[4];
        let stream_id = u32::from_be_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]) & 0x7FFFFFFF;

        let payload = bytes[9..].to_vec();

        Some(Http2Frame {
            length,
            type_,
            flags,
            stream_id,
            payload,
        })
    }
}

fn handle_http2_frame(frame: Http2Frame) {
    match frame.type_ {
        0x1 => println!("HEADERS frame received"),
        0x0 => println!("DATA frame received"),
        0x4 => println!("SETTINGS frame received"),
        _ => println!("Unknown frame type received"),
    }

    // Handle the frame based on its type
    // For example, if it's a HEADERS frame, parse the headers and prepare a response
}

fn handle_client_http2_v0001(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    if let Some(frame) = Http2Frame::from_bytes(&buffer) {
        handle_http2_frame(frame);
    } else {
        eprintln!("Failed to parse HTTP/2 frame");
    }

    // Send a basic HTTP/2 response (this is just a placeholder)
    let response = b"HTTP/2 200 OK\r\nContent-Length: 12\r\n\r\nHello, world!";
    stream.write(response).unwrap();
    stream.flush().unwrap();
}

fn handle_client_http2(mut stream: TcpStream) {
    handle_connection_preface(&mut stream);

    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    if let Some(frame) = Http2Frame::from_bytes(&buffer) {
        handle_http2_frame(frame);
    } else {
        eprintln!("Failed to parse HTTP/2 frame");
    }

    // Send a basic HTTP/2 response (this is just a placeholder)
    let response = b"HTTP/2 200 OK\r\nContent-Length: 12\r\n\r\nHello, world!";
    stream.write(response).unwrap();
    stream.flush().unwrap();
}

fn handle_connection_preface(stream: &mut TcpStream) {
    let mut buffer = [0; 24];
    stream.read_exact(&mut buffer).unwrap();

    let preface = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
    if &buffer[..24] == preface {
        println!("Received valid HTTP/2 connection preface");
    } else {
        eprintln!("Invalid HTTP/2 connection preface");
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    // Here you would parse the HTTP/2 frames and handle the request
    // For now, we'll just send a simple response
    let response = b"HTTP/1.1 200 OK\r\nContent-Length: 12\r\n\r\nHello, world!";
    stream.write(response).unwrap();
    stream.flush().unwrap();
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_client(stream);
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}
