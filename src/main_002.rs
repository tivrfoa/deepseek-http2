use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

// Constants for frame types
const SETTINGS_FRAME_TYPE: u8 = 0x04;
const HEADERS_FRAME_TYPE: u8 = 0x01;
const WINDOW_UPDATE_FRAME_TYPE: u8 = 0x08;

// Frame header structure
struct FrameHeader {
    length: u32,
    type_: u8,
    flags: u8,
    stream_id: u32,
}

impl FrameHeader {
    fn from_bytes(bytes: &[u8; 9]) -> Self {
        let length = u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]]);
        let type_ = bytes[3];
        let flags = bytes[4];
        let stream_id = u32::from_be_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]) & 0x7FFFFFFF;

        FrameHeader {
            length,
            type_,
            flags,
            stream_id,
        }
    }
}

fn handle_connection_preface(stream: &mut TcpStream) -> bool {
    let mut preface_buffer = [0; 24];
    if let Err(_) = stream.read_exact(&mut preface_buffer) {
        eprintln!("Failed to read connection preface");
        return false;
    }

    let expected_preface = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
    if &preface_buffer[..] == expected_preface {
        println!("Valid HTTP/2 connection preface received");
        true
    } else {
        eprintln!("Invalid HTTP/2 connection preface");
        false
    }
}

fn send_http2_settings_frame(stream: &mut TcpStream) {
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

fn read_client_settings_frame(stream: &mut TcpStream) -> bool {
    let mut header_buffer = [0; 9];
    if let Err(_) = stream.read_exact(&mut header_buffer) {
        eprintln!("Failed to read frame header");
        return false;
    }

    let header = FrameHeader::from_bytes(&header_buffer);

    // Check if this is a SETTINGS frame
    if header.type_ != SETTINGS_FRAME_TYPE {
        eprintln!("Expected SETTINGS frame, got frame type {}", header.type_);
        return false;
    }

    println!(
        "Received SETTINGS frame: length={}, flags={}, stream_id={}",
        header.length, header.flags, header.stream_id
    );

    // Read the payload (if any)
    if header.length > 0 {
        let mut payload = vec![0; header.length as usize];
        if let Err(_) = stream.read_exact(&mut payload) {
            eprintln!("Failed to read frame payload");
            return false;
        }

        // Parse the settings
        for chunk in payload.chunks(6) {
            if chunk.len() == 6 {
                let key = u16::from_be_bytes([chunk[0], chunk[1]]);
                let value = u32::from_be_bytes([chunk[2], chunk[3], chunk[4], chunk[5]]);
                println!("Setting: key={}, value={}", key, value);
            }
        }
    }

    // Send a SETTINGS acknowledgment
    let ack_frame = [
        0x00, 0x00, 0x00, // Length: 0 (empty payload)
        0x04,             // Type: SETTINGS (4)
        0x01,             // Flags: ACK (0x01)
        0x00, 0x00, 0x00, 0x00, // Stream ID: 0 (connection-level)
    ];

    if let Err(_) = stream.write_all(&ack_frame) {
        eprintln!("Failed to send SETTINGS acknowledgment");
        return false;
    }

    true
}

fn read_window_update_frame(stream: &mut TcpStream) -> bool {
    let mut header_buffer = [0; 9];
    if let Err(_) = stream.read_exact(&mut header_buffer) {
        eprintln!("Failed to read frame header");
        return false;
    }

    let header = FrameHeader::from_bytes(&header_buffer);

    // Check if this is a WINDOW_UPDATE frame
    if header.type_ != WINDOW_UPDATE_FRAME_TYPE {
        eprintln!("Expected WINDOW_UPDATE frame, got frame type {}", header.type_);
        return false;
    }

    println!(
        "Received WINDOW_UPDATE frame: length={}, flags={}, stream_id={}",
        header.length, header.flags, header.stream_id
    );

    // Read the payload (if any)
    if header.length > 0 {
        let mut payload = vec![0; header.length as usize];
        if let Err(_) = stream.read_exact(&mut payload) {
            eprintln!("Failed to read frame payload");
            return false;
        }

        // Parse the window size increment
        let increment = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
        println!("Window size increment: {}", increment);
    }

    true
}

fn read_headers_frame(stream: &mut TcpStream) -> bool {
    let mut header_buffer = [0; 9];
    if let Err(_) = stream.read_exact(&mut header_buffer) {
        eprintln!("Failed to read frame header");
        return false;
    }

    let header = FrameHeader::from_bytes(&header_buffer);

    // Check if this is a HEADERS frame
    if header.type_ != HEADERS_FRAME_TYPE {
        eprintln!("Expected HEADERS frame, got frame type {}", header.type_);
        return false;
    }

    println!(
        "Received HEADERS frame: length={}, flags={}, stream_id={}",
        header.length, header.flags, header.stream_id
    );

    // Read the payload (if any)
    if header.length > 0 {
        let mut payload = vec![0; header.length as usize];
        if let Err(_) = stream.read_exact(&mut payload) {
            eprintln!("Failed to read frame payload");
            return false;
        }

        // For simplicity, assume the payload contains raw headers (not HPACK-compressed)
        let headers = String::from_utf8_lossy(&payload);
        println!("Headers: {}", headers);
    }

    true
}

fn send_response(stream: &mut TcpStream) {
    // Send a HEADERS frame with the response headers
    let headers_frame = [
        0x00, 0x00, 0x1D, // Length: 29 bytes (for the headers below)
        0x01,             // Type: HEADERS (1)
        0x04,             // Flags: END_HEADERS (0x04)
        0x00, 0x00, 0x00, 0x01, // Stream ID: 1 (client's request stream)
        // Headers (simplified for demonstration)
        b':', b's', b't', b'a', b't', b'u', b's', b' ', b'2', b'0', b'0', b' ', b'\r', b'\n',
        b'c', b'o', b'n', b't', b'e', b'n', b't', b'-', b'l', b'e', b'n', b'g', b't', b'h', b' ',
        b'1', b'2', b'\r', b'\n', b'\r', b'\n',
    ];

    stream.write_all(&headers_frame).unwrap();

    // Send a DATA frame with the response body
    let data_frame_header = [
        0x00, 0x00, 0x0C, // Length: 12 bytes (for the body below)
        0x00,             // Type: DATA (0)
        0x01,             // Flags: END_STREAM (0x01)
        0x00, 0x00, 0x00, 0x01, // Stream ID: 1 (client's request stream)
    ];

    stream.write_all(&data_frame_header).unwrap();
    stream.write_all(b"Hello, world!").unwrap();
    stream.flush().unwrap();
}

fn handle_client(mut stream: TcpStream) {
    // Step 1: Read and validate the HTTP/2 connection preface
    if !handle_connection_preface(&mut stream) {
        return; // Close the connection if the preface is invalid
    }

    // Step 2: Send the server's SETTINGS frame
    send_http2_settings_frame(&mut stream);

    // Step 3: Read the client's SETTINGS frame
    if !read_client_settings_frame(&mut stream) {
        return; // Close the connection if the frame is invalid
    }

    // Step 4: Handle additional frames (e.g., WINDOW_UPDATE)
    loop {
        let mut header_buffer = [0; 9];
        if let Err(_) = stream.read_exact(&mut header_buffer) {
            eprintln!("Failed to read frame header");
            return;
        }

        let header = FrameHeader::from_bytes(&header_buffer);

        match header.type_ {
            WINDOW_UPDATE_FRAME_TYPE => {
                if !read_window_update_frame(&mut stream) {
                    return; // Close the connection if the frame is invalid
                }
            }
            HEADERS_FRAME_TYPE => {
                if !read_headers_frame(&mut stream) {
                    return; // Close the connection if the frame is invalid
                }
                break; // Exit the loop after processing the HEADERS frame
            }
            _ => {
                eprintln!("Unexpected frame type: {}", header.type_);
                return; // Close the connection on unexpected frame types
            }
        }
    }

    // Step 5: Send a response
    send_response(&mut stream);
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(|| {
                    handle_client(stream);
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}
