

# I am getting 18 for frame.length

If you're getting a `frame.length` of **18** for the `SETTINGS` frame sent by `curl`, it means `curl` is including a **payload** in the `SETTINGS` frame. This payload contains specific HTTP/2 settings that `curl` wants to negotiate with the server.

---

### **Why is the Payload Length 18?**
In HTTP/2, each setting in the `SETTINGS` frame is a **key-value pair** that is 6 bytes long:
- **Key**: 2 bytes (identifies the setting, e.g., `SETTINGS_MAX_CONCURRENT_STREAMS`).
- **Value**: 4 bytes (the value for the setting).

A payload length of **18 bytes** means there are **3 settings** in the frame (since \( 3 \times 6 = 18 \)).

---

### **Common Settings in `curl`'s `SETTINGS` Frame**
Here are some common settings that `curl` might include:
1. **`SETTINGS_HEADER_TABLE_SIZE` (0x1)**:
   - Specifies the maximum size of the header compression table.
   - Default value: **4096 bytes**.

2. **`SETTINGS_ENABLE_PUSH` (0x2)**:
   - Indicates whether server push is enabled.
   - Default value: **1** (enabled).

3. **`SETTINGS_MAX_CONCURRENT_STREAMS` (0x3)**:
   - Specifies the maximum number of concurrent streams.
   - Default value: **100**.

---

### **Parsing the Payload**
To parse the payload, you need to:
1. Read the 18-byte payload.
2. Split it into 6-byte chunks (each representing a setting).
3. Decode the key and value for each setting.

Here’s how you can update your `read_client_settings_frame` function to handle this:

```rust
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
```

---

### **Example Output**
If `curl` sends a `SETTINGS` frame with a payload length of 18, the server might print something like this:
```
Received SETTINGS frame: length=18, flags=0, stream_id=0
Setting: key=1, value=4096
Setting: key=2, value=1
Setting: key=3, value=100
```

---

### **What Each Setting Means**
1. **`key=1, value=4096`**:
   - `SETTINGS_HEADER_TABLE_SIZE` is set to **4096 bytes**.

2. **`key=2, value=1`**:
   - `SETTINGS_ENABLE_PUSH` is set to **1** (server push is enabled).

3. **`key=3, value=100`**:
   - `SETTINGS_MAX_CONCURRENT_STREAMS` is set to **100**.

---

### **Next Steps**
1. **Acknowledge the `SETTINGS` Frame**:
   - The server must send a `SETTINGS` frame with the `ACK` flag set, as shown in the code above.

2. **Apply the Settings**:
   - Update the server's configuration based on the received settings (e.g., adjust the maximum number of concurrent streams).

3. **Proceed with the Request**:
   - After acknowledging the `SETTINGS` frame, the server should expect a `HEADERS` frame from the client containing the HTTP request.

---

### **Testing with `curl`**
Run the server and test it with:
```bash
curl -v --http2-prior-knowledge http://localhost:8080
```

If everything is implemented correctly, the server should:
1. Read the connection preface.
2. Send its `SETTINGS` frame.
3. Read the client's `SETTINGS` frame (with `length = 18`).
4. Parse and print the settings.
5. Send a `SETTINGS` acknowledgment.
6. Respond to the client's request.

Let me know if you have further questions!

# Settings key/pair


```
I got this:

Setting: key=3, value=100
Setting: key=4, value=10485760
Setting: key=2, value=0
```

The output you received indicates that `curl` sent a `SETTINGS` frame with three settings:

1. **`key=3, value=100`**:
   - **Setting**: `SETTINGS_MAX_CONCURRENT_STREAMS` (0x3)
   - **Meaning**: The client is indicating that it supports a maximum of **100 concurrent streams** (HTTP/2 requests) on this connection.

2. **`key=4, value=10485760`**:
   - **Setting**: `SETTINGS_INITIAL_WINDOW_SIZE` (0x4)
   - **Meaning**: The client is setting the **initial flow-control window size** to **10,485,760 bytes** (10 MB). This controls how much data the client can receive before applying flow control.

3. **`key=2, value=0`**:
   - **Setting**: `SETTINGS_ENABLE_PUSH` (0x2)
   - **Meaning**: The client is **disabling server push** by setting this value to **0**.

---

### **What This Means for Your Server**
1. **`SETTINGS_MAX_CONCURRENT_STREAMS` (key=3, value=100)**:
   - Your server should respect this limit and not attempt to open more than 100 concurrent streams on this connection.

2. **`SETTINGS_INITIAL_WINDOW_SIZE` (key=4, value=10485760)**:
   - Your server should use this value as the initial flow-control window size for all streams. This means the server can send up to 10 MB of data to the client before waiting for a `WINDOW_UPDATE` frame.

3. **`SETTINGS_ENABLE_PUSH` (key=2, value=0)**:
   - Your server should **not send any server push promises** to this client, as the client has explicitly disabled server push.

---

### **How to Handle These Settings**
1. **Acknowledge the `SETTINGS` Frame**:
   - Your server must send a `SETTINGS` frame with the `ACK` flag set to acknowledge the client's settings. This is already implemented in the code I provided earlier.

2. **Apply the Settings**:
   - Update your server's internal state to reflect the client's settings. For example:
     - Track the maximum number of concurrent streams.
     - Use the initial window size for flow control.
     - Disable server push if the client has disabled it.

3. **Proceed with the Request**:
   - After acknowledging the `SETTINGS` frame, the server should expect a `HEADERS` frame from the client containing the HTTP request.

---

### **Updated Code to Handle These Settings**
Here’s how you can update your server to handle these settings:

```rust
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

                // Apply the settings
                match key {
                    3 => {
                        println!("Client supports a maximum of {} concurrent streams", value);
                        // Update the server's maximum concurrent streams limit
                    }
                    4 => {
                        println!("Client's initial window size is {} bytes", value);
                        // Update the server's flow-control window size
                    }
                    2 => {
                        if value == 0 {
                            println!("Client has disabled server push");
                            // Disable server push for this connection
                        }
                    }
                    _ => {
                        println!("Unknown setting: key={}, value={}", key, value);
                    }
                }
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
```

---

### **Example Output**
With the updated code, your server should now print:
```
Received SETTINGS frame: length=18, flags=0, stream_id=0
Setting: key=3, value=100
Client supports a maximum of 100 concurrent streams
Setting: key=4, value=10485760
Client's initial window size is 10485760 bytes
Setting: key=2, value=0
Client has disabled server push
```

---

### **Next Steps**
1. **Respect the Client's Settings**:
   - Ensure your server adheres to the client's settings (e.g., do not exceed 100 concurrent streams, respect the initial window size, and disable server push).

2. **Handle the Request**:
   - After acknowledging the `SETTINGS` frame, the server should expect a `HEADERS` frame from the client containing the HTTP request.

3. **Test with `curl`**:
   - Run the server and test it with:
     ```bash
     curl -v --http2-prior-knowledge http://localhost:8080
     ```

Let me know if you need further assistance!

# Asking to implement HEADERS frame

Implement this part:

After acknowledging the SETTINGS frame, the server should expect a HEADERS frame from the client containing the HTTP request.

To implement the handling of the `HEADERS` frame after acknowledging the `SETTINGS` frame, we need to:

1. **Read the `HEADERS` Frame**:
   - The `HEADERS` frame contains the HTTP request headers sent by the client.
   - It has the same 9-byte header format as other HTTP/2 frames, followed by a payload containing the headers.

2. **Parse the `HEADERS` Frame**:
   - The payload of the `HEADERS` frame contains the HTTP headers in **HPACK-compressed format**. For simplicity, we'll assume the headers are uncompressed (which is not realistic but works for demonstration purposes).

3. **Send a Response**:
   - After parsing the `HEADERS` frame, the server should send an HTTP/2 response (e.g., a `HEADERS` frame with the response headers and a `DATA` frame with the response body).

---

### **Updated Server Code**

Here’s how you can update your server to handle the `HEADERS` frame:

```rust
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

// Constants for frame types
const SETTINGS_FRAME_TYPE: u8 = 0x04;
const HEADERS_FRAME_TYPE: u8 = 0x01;

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

    // Step 4: Read the client's HEADERS frame
    if !read_headers_frame(&mut stream) {
        return; // Close the connection if the frame is invalid
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
```

---

### **Key Points**
1. **`HEADERS` Frame Handling**:
   - The server reads the `HEADERS` frame and prints the headers (assuming they are uncompressed for simplicity).

2. **Response**:
   - The server sends a `HEADERS` frame with the response headers and a `DATA` frame with the response body.

3. **Stream ID**:
   - The `HEADERS` and `DATA` frames use the same stream ID as the client's request (stream ID 1 in this case).

---

### **Testing with `curl`**
Run the server and test it with:
```bash
curl -v --http2-prior-knowledge http://localhost:8080
```

If everything is implemented correctly, the server should:
1. Read the connection preface.
2. Send its `SETTINGS` frame.
3. Read the client's `SETTINGS` frame.
4. Read the client's `HEADERS` frame.
5. Send a response with `Hello, world!`.

Let me know if you need further assistance!

# Expected HEADERS frame, got frame type 8

The error `Expected HEADERS frame, got frame type 8` indicates that the server received a frame of type **8** (`WINDOW_UPDATE`) instead of the expected `HEADERS` frame. This is normal behavior in HTTP/2 because clients often send a `WINDOW_UPDATE` frame to adjust the flow-control window size before sending the `HEADERS` frame.

---

### **Why is a `WINDOW_UPDATE` Frame Sent?**
In HTTP/2, **flow control** is managed using the `WINDOW_UPDATE` frame. The client may send a `WINDOW_UPDATE` frame to increase the flow-control window size, allowing the server to send more data.

---

### **How to Handle This**
To handle this correctly, the server should:
1. **Read and Acknowledge the `WINDOW_UPDATE` Frame**:
   - Parse the `WINDOW_UPDATE` frame and update the flow-control window size.
2. **Continue Waiting for the `HEADERS` Frame**:
   - After handling the `WINDOW_UPDATE` frame, the server should continue waiting for the `HEADERS` frame.

---

### **Updated Code to Handle `WINDOW_UPDATE` Frames**

Here’s how you can update your server to handle `WINDOW_UPDATE` frames:

```rust
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
```

---

### **Key Changes**
1. **Added `WINDOW_UPDATE` Frame Handling**:
   - The server now reads and processes `WINDOW_UPDATE` frames before expecting the `HEADERS` frame.

2. **Loop to Handle Multiple Frames**:
   - The server enters a loop to handle additional frames (e.g., `WINDOW_UPDATE`) until it receives the `HEADERS` frame.

3. **Break on `HEADERS` Frame**:
   - The loop exits when the `HEADERS` frame is received, allowing the server to proceed with sending a response.

---

### **Testing with `curl`**
Run the server and test it with:
```bash
curl -v --http2-prior-knowledge http://localhost:8080
```

If everything is implemented correctly, the server should:
1. Read the connection preface.
2. Send its `SETTINGS` frame.
3. Read the client's `SETTINGS` frame.
4. Read and handle any `WINDOW_UPDATE` frames.
5. Read the client's `HEADERS` frame.
6. Send a response with `Hello, world!`.

Let me know if you need further assistance!
