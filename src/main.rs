use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

/*
Deepseek:

v 0.0.1:

Conclusion

This is a very basic starting point for an HTTP/2 server in Rust. A fully functional HTTP/2 server would require handling many more details, such as flow control, stream prioritization, header compression (HPACK), and more. Implementing all of this from scratch is a significant undertaking, which is why most developers use existing libraries like hyper or h2 for HTTP/2 in Rust.

If you want to build a production-ready HTTP/2 server, I highly recommend using these libraries, as they handle all the complexities of the protocol for you.

v 0.0.2

Order of Operations

  1. Server Reads the Preface:
    1.2. The server must first read and validate the 24-byte preface sent by the client.
    1.3 If the preface is invalid, the server should close the connection.
  2. Server Sends Its SETTINGS Frame:
    2.1 After validating the preface, the server must send its own SETTINGS frame.
  3. Server Reads the Client's SETTINGS Frame:
    3.1 The server must then read the client's SETTINGS frame (sent after the preface).
  4. Server Sends a SETTINGS Acknowledgment:
    4.1 The server must acknowledge the client's SETTINGS frame by sending a SETTINGS
    frame with the ACK flag set.

*/

#[derive(Debug)]
struct Http2Frame {
    length: u32,
    type_: u8,
    flags: u8,
    stream_id: u32,
    payload: Vec<u8>,
}

impl Http2Frame {
    fn from_bytes(bytes: &[u8; 9]) -> Self {
        let length = u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]]);
        let type_ = bytes[3];
        let flags = bytes[4];
        let stream_id = u32::from_be_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]) & 0x7FFFFFFF;

        Http2Frame {
            length,
            type_,
            flags,
            stream_id,
            payload: vec![],
        }
    }
}

fn read_client_settings_frame(stream: &mut TcpStream) -> bool {
    println!("----- read_client_settings_frame");

    let mut header_buffer = [0; 9];
    if let Err(e) = stream.read_exact(&mut header_buffer) {
        eprintln!("[ERROR] failed to read frame header: {e}");
        return false;
    }
    println!("[INFO] read frame header");

    let mut frame = Http2Frame::from_bytes(&header_buffer);
    dbg!(frame.length, frame.type_, frame.flags, frame.stream_id,
            &frame.payload);
    match frame.type_ {
        0x1 => println!("HEADERS frame received"),
        0x0 => println!("DATA frame received"),
        0x4 => println!("SETTINGS frame received"),
        _ => println!("Unknown frame type received"),
    }

    // TODO: Handle the frame based on its type
    // For example, if it's a HEADERS frame, parse the headers and prepare a response

    // Read the payload (if any)
    if frame.length > 0 {
        println!("[TRACE] will try to read payload of size: {}", frame.length);
        let mut payload = vec![0; frame.length as usize];
        if let Err(_) = stream.read_exact(&mut payload) {
            eprintln!("Failed to read frame payload");
            return false;
        }
        println!("SETTINGS payload: {:?}", payload);
    }

    true
}

fn handle_client_http2(mut stream: TcpStream) {
    println!("----- handle_client_http2");
    handle_connection_preface(&mut stream);

    // Step 2: Send the server's SETTINGS frame
    send_http2_settings_frame(&mut stream);

    read_client_settings_frame(&mut stream);

    println!("[INFO] writing response");
    // Send a basic HTTP/2 response (this is just a placeholder)
    let response = b"HTTP/2 200 OK\r\nContent-Length: 12\r\n\r\nHello, world!";
    stream.write(response).unwrap();
    stream.flush().unwrap();
}

fn handle_connection_preface(stream: &mut TcpStream) {
    println!("----- handle_connection_preface");
    let mut buffer = [0; 24];
    stream.read_exact(&mut buffer).unwrap();
    let preface_req = String::from_utf8(buffer.to_vec()).unwrap();
    println!("preface: {preface_req}");

    let preface = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
    if &buffer[..24] == preface {
        println!("Received valid HTTP/2 connection preface");
    } else {
        eprintln!("Invalid HTTP/2 connection preface");
    }
}

fn send_http2_settings_frame(stream: &mut TcpStream) {
    println!("----- send_http2_settings_frame");
    // HTTP/2 SETTINGS frame (empty payload for simplicity)
    let settings_frame = [
        0x00, 0x00, 0x00, // Length: 0 (empty payload)
        0x04,             // Type: SETTINGS (4)
        0x00,             // Flags: None
        0x00, 0x00, 0x00, 0x00, // Stream ID: 0 (connection-level)
    ];

    stream.write_all(&settings_frame).unwrap();
    stream.flush().unwrap();
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    // handle_client_http1(stream);
                    handle_client_http2(stream);
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}

fn handle_client_http1(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    // Here you would parse the HTTP/2 frames and handle the request
    // For now, we'll just send a simple response
    let response = b"HTTP/1.1 200 OK\r\nContent-Length: 12\r\n\r\nHello, world!";
    stream.write(response).unwrap();
    stream.flush().unwrap();
}
