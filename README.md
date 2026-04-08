# MeshRelay

Encrypted messaging through a relay network. Works even when the recipient does not have the app installed.

[![Status](https://img.shields.io/badge/status-foundation-81a1c1?style=for-the-badge)]()
[![Privacy](https://img.shields.io/badge/privacy-E2E_encrypted-bd93f9?style=for-the-badge)](#trust-model)
[![License](https://img.shields.io/badge/license-MIT-f1fa8c?style=for-the-badge)](LICENSE)

[Architecture](#architecture) · [Quick Start](#quick-start) · [Run a Relay](#run-a-relay) · [Web Reader](#web-reader) · [Security](#trust-model)

---

## The Problem

Every mesh messenger requires both parties to have the app installed. Nobody joins a network where they cannot message anyone yet.

## How It Works

Three modes:

1. **Relay mode.** Encrypted messages stored on relay nodes. Recipient fetches when they come online. Neither party needs to be online at the same time.
2. **Web reader.** Non-users receive a link via SMS or email. They open it in any browser, the message decrypts locally, they can reply. No app install needed.
3. **Local mesh.** When internet is down, devices sync directly via Bluetooth or WiFi Direct.

A handful of volunteer relays (Raspberry Pi or VPS) route messages globally. Relays only handle ciphertext and cannot read anything.

```
Alice (app) --encrypted--> Relay --encrypted--> Bob (app)
                                        |
                                        v
                                Carol (no app)
                                receives link, opens in browser
```

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
| POST | /api/v1/register | Register a public key |
| POST | /api/v1/messages | Store encrypted message for recipient |
| GET | /api/v1/messages/{id} | Fetch pending messages |
| POST | /api/v1/messages/{id}/ack | Acknowledge delivery |
| POST | /api/v1/relay/forward | Relay-to-relay forwarding |
| GET | /api/v1/health | Health check |

## Quick Start

### Run a Relay

```bash
cd relay-daemon
cp config.example.toml config.toml
cargo run --release
```

The relay starts on 0.0.0.0:8080 with a SQLite database.

### Store a Message

```bash
curl -X POST http://localhost:8080/api/v1/messages \
  -H "Content-Type: application/json" \
  -d '{
    "recipient_id": "user-uuid",
    "sender_id": "sender-uuid",
    "ciphertext": "base64-encrypted-data",
    "sender_ratchet_key": "hex-encoded-ratchet-key",
    "msg_num": 1
  }'
```

### Fetch Messages

```bash
curl http://localhost:8080/api/v1/messages/{recipient_id}
```

### Web Reader

```
https://relay.meshtouch.link/r/{message_id}#key={encryption_key}&iv={iv}
```

The encryption key is in the URL fragment (after #). Browsers never send the fragment to the server, so the relay only sees the message ID.

## Trust Model

The relay handles ciphertext only. It never sees plaintext content, encryption keys, contact lists, or user identities beyond public key fingerprints.

The web reader performs client-side decryption using the Web Crypto API. The encryption key is in the URL fragment which browsers never send to servers.

## Run a Relay on Raspberry Pi

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

One Raspberry Pi 4 handles about 1,000 users and 10K messages per day.

## Roadmap

| Phase | Status | What |
|-------|--------|------|
| Relay daemon, web reader, SQLite | Done | Foundation |
| Mobile app with relay and Bluetooth mesh | Planned | Android |
| SMS gateway for non-user onboarding | Planned | Twilio/SignalWire bridge |
| Public relay registry, peering protocol | Planned | Relay network |
| Push notifications (metadata only) | Planned | FCM/APNs |
| Contact discovery (phone number to public key) | Planned | Opt-in |

---

[MIT License](LICENSE)
