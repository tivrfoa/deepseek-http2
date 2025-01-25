

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
