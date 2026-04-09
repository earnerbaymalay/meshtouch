# MeshRelay usage guide

Instructions for setting up and using MeshRelay.

---

## Quick start

### Run a relay

1.  Navigate to the `relay-daemon` directory:
    ```bash
    cd relay-daemon
    ```

2.  Copy the example configuration:
    ```bash
    cp config.example.toml config.toml
    ```

3.  Run the relay server:
    ```bash
    cargo run --release
    ```

    The relay will start on `0.0.0.0:8080` and utilize a SQLite database for message storage.

### Store a message

To store an encrypted message on the relay:

```bash
curl -X POST http://localhost:8080/api/v1/messages 
  -H "Content-Type: application/json" 
  -d '{
    "recipient_id": "user-uuid",
    "sender_id": "sender-uuid",
    "ciphertext": "base64-encrypted-data",
    "sender_ratchet_key": "hex-encoded-ratchet-key",
    "msg_num": 1
  }'
```

### Fetch messages

To retrieve pending messages for a recipient:

```bash
curl http://localhost:8080/api/v1/messages/{recipient_id}
```

### Web reader

Non-users can access messages via a secure link:

```
https://relay.meshtouch.link/r/{message_id}#key={encryption_key}&iv={iv}
```

The encryption key is embedded in the URL fragment (after the `#`), which is not sent to the server by browsers, ensuring privacy.

---

## Run a relay on Raspberry Pi

1.  Install build tools and Rust:
    ```bash
    sudo apt install build-essential pkg-config libssl-dev
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source ~/.cargo/env
    ```

2.  Clone the repository and build the relay:
    ```bash
    git clone https://github.com/earnerbaymalay/meshtouch.git
    cd meshtouch/relay-daemon
    cargo build --release
    cp config.example.toml config.toml
    ```

3.  Set up as a systemd service:
    ```bash
    sudo cp meshtouch.service /etc/systemd/system/
    sudo systemctl enable meshtouch
    sudo systemctl start meshtouch
    ```

    A single Raspberry Pi 4 can support approximately 1,000 users and 10,000 messages daily.

---

## Troubleshooting

See the separate `TROUBLESHOOTING.md` for common issues and solutions.

---

[MIT License](LICENSE)
