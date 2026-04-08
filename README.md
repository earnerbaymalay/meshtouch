<div align="center">

# 🔗 MeshRelay

### *Encrypted messaging that works even when the other person doesn't have the app.*

<p align="center">
  <a href="https://earnerbaymalay.github.io/sideload/">
    📲 <strong>Part of the local-first ecosystem: Sideload Hub</strong>
  </a>
</p>

[![License](https://img.shields.io/badge/license-MIT-f1fa8c?style=for-the-badge)](LICENSE)
[![Status](https://img.shields.io/badge/status-foundation-81a1c1?style=for-the-badge)]()
[![Privacy](https://img.shields.io/badge/privacy-E2E_encrypted-bd93f9?style=for-the-badge)](#trust-model)

[Architecture](#architecture) · [Quick Start](#quick-start) · [Run a Relay](#run-a-relay) · [Web Reader](#web-reader) · [Security](#trust-model)

</div>

---

## The Problem

Every mesh messenger (FireChat, Briar, Bridgefy) has the same fatal flaw: **both parties must have the app installed**. This creates an adoption death spiral — nobody joins a network where they can't message anyone yet.

## The Solution

MeshRelay uses a **hybrid relay architecture** that decouples messaging from app adoption:

1. **Normal mode**: Encrypted messages stored on relay nodes. Recipient fetches when they come online. No simultaneous presence needed.
2. **Web reader**: Non-users receive SMS/email with a link. They open it in any browser → message decrypts locally → they can reply. Zero install needed.
3. **Local mesh**: When internet is down, devices sync directly via Bluetooth/WiFi Direct.
4. **Relay network**: A handful of always-on volunteer relays (Raspberry Pi / VPS) route messages globally.

```
Alice (app) ──encrypted──► Relay ──encrypted──► Bob (app)
                                      │
                                      ▼
                              Carol (no app)
                              receives SMS with link
                              opens in browser → reads
```

## Architecture

```
meshtouch/
├── relay-daemon/     # Rust relay server (stores encrypted messages)
├── mobile-app/       # Android client (Kotlin, built on e2eecc crypto)
├── web-reader/       # Browser-based reader for non-users (HTML/JS)
├── sms-gateway/      # SMS/Email bridge (Python)
└── docs/             # Setup guides, security docs
```

### Relay API

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/v1/register` | Register a public key |
| `POST` | `/api/v1/messages` | Store encrypted message for recipient |
| `GET` | `/api/v1/messages/{id}` | Fetch pending messages |
| `POST` | `/api/v1/messages/{id}/ack` | Acknowledge delivery |
| `POST` | `/api/v1/relay/forward` | Relay-to-relay forwarding |
| `GET` | `/api/v1/health` | Health check |

## Quick Start

### Run a Relay

```bash
cd relay-daemon
cp config.example.toml config.toml  # Edit with your settings
cargo run --release
```

The relay starts on `0.0.0.0:8080` with a SQLite database.

### Open the Web Reader

```
https://relay.meshtouch.link/r/{message_id}#key={encryption_key}&iv={iv}
```

The encryption key is in the URL **fragment** (`#...`) — it's **never sent to the server**. The relay only sees the message ID.

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

## Trust Model

The relay **only handles ciphertext**. It never sees:

| Relay Can See | Relay Cannot See |
|---------------|-----------------|
| Message sizes | Plaintext content |
| Timing patterns | Encryption keys |
| Connection IPs | Contact lists |
| Public key fingerprints | User identities |

The web reader performs **client-side decryption** using the Web Crypto API. The encryption key is in the URL fragment, which browsers **never send to servers**.

## Run a Relay (Raspberry Pi)

```bash
# On a Raspberry Pi 4 running Raspberry Pi OS
sudo apt install build-essential pkg-config libssl-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

git clone https://github.com/earnerbaymalay/meshtouch.git
cd meshtouch/relay-daemon
cargo build --release

# Create config
cp config.example.toml config.toml

# Run as a service
sudo cp meshtouch.service /etc/systemd/system/
sudo systemctl enable meshtouch
sudo systemctl start meshtouch
```

One Pi can handle ~1,000 users and ~10K messages/day with minimal resources.

## Roadmap

| Phase | Status | What |
|-------|--------|------|
| **Foundation** | ✅ | Relay daemon, web reader, SQLite storage |
| **Mobile App** | 🔮 | Android client with relay + Bluetooth mesh |
| **SMS Gateway** | 🔮 | Twilio/SignalWire bridge for non-user onboarding |
| **Relay Network** | 🔮 | Public relay registry, peering protocol |
| **Push Notifications** | 🔮 | FCM/APNs for message alerts (metadata only) |
| **Contact Discovery** | 🔮 | Phone number → public key lookup (opt-in) |

## Ecosystem

MeshRelay is part of the local-first app ecosystem:

| Project | Repo | Purpose |
|---------|------|---------|
| 🌌 Aether | [aether](https://github.com/earnerbaymalay/aether) | Local AI workstation |
| 🔗 MeshRelay | (this repo) | Encrypted messaging with relay network |
| 🛡️ Cypherchat | [e2eecc](https://github.com/earnerbaymalay/e2eecc) | E2EE messaging (Double Ratchet crypto foundation) |
| 🌗 Gloam | [Gloam](https://github.com/earnerbaymalay/Gloam) | Solar-timed journaling |

📲 **[Install any app from the Sideload Hub →](https://earnerbaymalay.github.io/sideload/)**

---

<div align="center">

**[MIT License](LICENSE)** — Free forever. Use it. Modify it. Share it.

*Develop natively. Think locally. Evolve autonomously.*

</div>
