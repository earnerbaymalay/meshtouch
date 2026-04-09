# MeshRelay

Encrypted messaging through a relay network. It functions even when the recipient does not have the app installed.

[![Status](https://img.shields.io/badge/status-foundation-50fa7b?style=for-the-badge)]()
[![Privacy](https://img.shields.io/badge/privacy-E2E_encrypted-bd93f9?style=for-the-badge)](docs/SECURITY.md)
[![License](https://img.shields.io/badge/license-MIT-f1fa8c?style=for-the-badge)](LICENSE)

[Architecture](#architecture) · [Quick start](#quick-start) · [Run a relay](#run-a-relay) · [Web reader](#web-reader) · [Security](#trust-model)

---

## The problem

Traditional mesh messengers require both parties to have the app installed, hindering adoption for new users.

---

## How it works

MeshRelay operates in three modes:

1.  **Relay mode:** Encrypted messages are stored on relay nodes. The recipient retrieves them when they come online, eliminating the need for both parties to be simultaneously online.
2.  **Web reader:** Non-users receive a secure link via SMS or email. They can open this link in any browser to decrypt and read the message locally, and even reply, without needing to install an app.
3.  **Local mesh:** When internet connectivity is unavailable, devices can sync directly via Bluetooth or Wi-Fi Direct.

A network of volunteer relays (Raspberry Pi or VPS) routes messages globally. These relays handle only ciphertext and cannot access the plaintext content of messages.

```
Alice (app) --encrypted--> Relay --encrypted--> Bob (app)
                                        |
                                        v
                                Carol (no app)
                                receives link, opens in browser
```

---

## Architecture

```
meshtouch/
├── relay-daemon/     # Rust relay server (Axum, SQLite)
├── mobile-app/       # Android client (Kotlin, e2eecc crypto)
├── web-reader/       # Browser-based reader (HTML, Web Crypto)
├── sms-gateway/      # SMS/Email bridge (Python)
└── docs/
```

### Relay API

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST   | `/api/v1/register` | Register a public key. |
| POST   | `/api/v1/messages` | Store an encrypted message for a recipient. |
| GET    | `/api/v1/messages/{id}` | Fetch pending messages. |
| POST   | `/api/v1/messages/{id}/ack` | Acknowledge message delivery. |
| POST   | `/api/v1/relay/forward` | Forward messages between relays. |
| GET    | `/api/v1/health` | Health check endpoint. |

---

## Quick start

### Run a relay

```bash
cd relay-daemon
cp config.example.toml config.toml
cargo run --release
```

The relay will start on `0.0.0.0:8080` and use a SQLite database.

### Store a message

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

```bash
curl http://localhost:8080/api/v1/messages/{recipient_id}
```

### Web reader

```
https://relay.meshtouch.link/r/{message_id}#key={encryption_key}&iv={iv}
```

The encryption key is embedded in the URL fragment (after `#`). Browsers do not send the fragment to the server, so the relay only sees the message ID.

---

## Trust model

The relay exclusively handles ciphertext. It never accesses plaintext content, encryption keys, contact lists, or user identities beyond public key fingerprints.

The web reader performs client-side decryption using the Web Crypto API. The encryption key resides in the URL fragment, which browsers never transmit to servers.

---

## Run a relay on Raspberry Pi

```bash
sudo apt install build-essential pkg-config libssl-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

git clone https://github.com/earnerbaymalay/meshtouch.git
cd meshtouch/relay-daemon
cargo build --release
cp config.example.toml config.toml

sudo cp meshtouch.service /etc/systemd/system/
sudo systemctl enable meshtouch
sudo systemctl start meshtouch
```

A single Raspberry Pi 4 can manage approximately 1,000 users and 10,000 messages per day.

---

## Roadmap

| Phase | Status | Objective |
|-------|--------|-----------|
| Relay daemon, web reader, SQLite | Done | Foundation |
| Mobile app with relay and Bluetooth mesh | Planned | Android |
| SMS gateway for non-user onboarding | Planned | Twilio/SignalWire bridge |
| Public relay registry, peering protocol | Planned | Relay network |
| Push notifications (metadata only) | Planned | FCM/APNs |
| Contact discovery (phone number to public key) | Planned | Opt-in |

---

[MIT License](LICENSE)
