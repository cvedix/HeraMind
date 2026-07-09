<p align="center">
  <img src="web/public/logo-light.png" alt="HeraMind" width="400">
</p>

<h3 align="center">Edge AI Platform for IoT Automation</h3>

<p align="center">
  Rust-powered edge intelligence — connect devices, run AI agents, automate everything.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/License-Apache--2.0-blue.svg" alt="License">
  <a href="https://github.com/CVEDIX/HeraMind/releases/latest">
    <img src="https://img.shields.io/github/v/release/CVEDIX/HeraMind?color=informational&label=release" alt="Release">
  </a>
  <a href="https://github.com/CVEDIX/HeraMind/stargazers">
    <img src="https://img.shields.io/github/stars/CVEDIX/HeraMind?style=social" alt="Stars">
  </a>
  <a href="https://discord.gg/gkM7cc8gKb">
    <img src="https://img.shields.io/discord/0.svg?logo=discord&logoColor=ffffff&label=Discord&color=5865F2&link=https://discord.gg/gkM7cc8gKb" alt="Discord Community">
  </a>
  <img src="https://img.shields.io/github/last-commit/CVEDIX/HeraMind?label=last%20commit&color=success" alt="Last Commit">
  <img src="https://img.shields.io/badge/Rust-1.85+-orange.svg" alt="Rust">
  <img src="https://img.shields.io/badge/Platform-macOS%20%7C%20Windows%20%7C%20Linux%20%7C%20Server-informational.svg" alt="Platform">
</p>

<br/>

<div align="center">
  <img src="docs/img/dashboard-main.png" alt="Dashboard" width="800" style="border-radius: 8px; box-shadow: 0 4px 16px rgba(0,0,0,0.12);" />
  <br/><sub><b>Dashboard</b></sub>
</div>

<br/>

<div align="center">
  <table>
    <tr>
      <td align="center">
        <img src="docs/img/chat.png" alt="AI Chat" width="400" style="border-radius: 8px; box-shadow: 0 4px 16px rgba(0,0,0,0.12);" />
        <br/><sub><b>AI Chat</b></sub>
      </td>
      <td align="center">
        <img src="docs/img/devices.png" alt="Devices" width="400" style="border-radius: 8px; box-shadow: 0 4px 16px rgba(0,0,0,0.12);" />
        <br/><sub><b>Device Management</b></sub>
      </td>
      <td align="center">
        <img src="docs/img/mobile_web.png" alt="Mobile" width="180" style="border-radius: 8px; box-shadow: 0 4px 16px rgba(0,0,0,0.12);" />
        <br/><sub><b>Mobile Web</b></sub>
      </td>
    </tr>
  </table>
</div>

<br/>

## What is HeraMind?

HeraMind is an **edge-deployed AI platform** that brings intelligence to IoT. It runs LLM-powered agents directly on your hardware, connecting to devices via MQTT/BLE/Webhook, automating responses through a rule engine, and visualizing everything on real-time dashboards — all without relying on cloud services.

**Key idea**: Talk to your devices in natural language. The AI understands your intent, queries device states, creates automation rules, and takes action autonomously.

> 📚 **Full documentation is on the [HeraMind Docs](https://docs.cvedix.com/heramind).** This README is a quick overview — visit the docs for complete guides:
>
> - [What is HeraMind?](https://docs.cvedix.com/heramind) — product overview and concepts
> - [Five-Minute Quick Start](https://docs.cvedix.com/heramind) — get running fast
> - [Install & Setup](https://docs.cvedix.com/heramind) — deployment, desktop app, server, Docker
> - [Developer Guide](https://docs.cvedix.com/heramind) — API, extensions, integrations

### Why HeraMind?

- **Fully self-contained** — Embedded MQTT broker, redb storage, no external database or broker to install
- **Type-safe end-to-end** — Rust backend with compile-time guarantees; agent CLI commands dispatch in-process with structured data, no fragile string parsing
- **Crash-proof extensions** — Extensions run in isolated processes with capability-based permissions; a misbehaving extension never takes down the server
- **Cloud-optional** — Works 100% offline with local LLMs (Ollama), or connect cloud models when you need more power

## Features

### AI-Powered Intelligence
- **Natural Language Chat** — Conversational interface to query and control all connected devices
- **Autonomous Agents** — Scheduled AI agents that monitor, analyze, and act on device data independently
- **10+ LLM Backends** — Ollama, OpenAI, Anthropic, Google, xAI, Qwen, DeepSeek, GLM, MiniMax, and any OpenAI-compatible endpoint
- **Memory System** — Multi-tier memory (User, Knowledge, Procedures, Session) with automatic extraction and compression
- **Skill System** — YAML+Markdown skills that guide agent behavior for specific scenarios
- **Multimodal** — Image upload and visual analysis support

### Device Management
- **MQTT Protocol** — Primary device integration with embedded broker, mTLS, and CA certificate support
- **BLE Provisioning** — Zero-touch device setup via Bluetooth (Tauri native + Web Bluetooth)
- **HTTP/Webhook** — Flexible device adapter for REST-based devices
- **Auto-Discovery** — Automatic device detection, type registration, and AI-assisted onboarding
- **Command Queue** — Send control commands to devices with parameter validation and tracking
- **Custom Device Types** — Define device metrics and commands via JSON type definitions

### Automation
- **Rule Engine** — JSON-based rules with recursive conditions (comparison/range/logical) and actions (notify/execute/trigger_agent), with cooldown and `for_duration` debouncing
- **Data Transforms** — JavaScript-based data transformation for creating virtual metrics
- **Scheduled Agents** — Time-based and event-driven AI agent execution
- **Event Bus** — Pub/sub architecture for decoupled component communication

### Dashboards & Visualization
- **Drag-and-Drop Builder** — Visual dashboard editor with responsive grid layout
- **Rich Widgets** — Value cards, charts, gauges, tables, VLM vision components
- **Real-time Updates** — WebSocket/SSE for live data streaming to dashboards
- **Dashboard Sharing** — Public links with expiration for sharing dashboards
- **Custom Components** — Build and publish your own dashboard widgets

### Notification & Data Push
- **7 Notification Channels** — Webhook, Email, Telegram, WeCom, DingTalk, Slack, Feishu
- **Data Push** — Forward telemetry data to external systems via Webhook or MQTT
- **Delivery Tracking** — Retry logic with exponential backoff, delivery history, and log management
- **Message Deduplication** — Prevent notification storms from high-frequency triggers

### Platform
- **Multi-Instance** — Connect to and manage multiple HeraMind backends from a single interface
- **Extension System** — Native & WASM extensions with process isolation and capability-based permissions
- **Cross-Platform Desktop** — macOS, Windows, Linux native apps via Tauri
- **Mobile-Friendly Web** — Responsive web UI optimized for phone and tablet
- **i18n** — Vietnamese, English, and Chinese language support
- **Dark Mode** — System-aware dark/light theme
- **API Key Auth** — Alternative to JWT for programmatic access
- **CLI Tools** — Full-featured command-line interface for all operations

## Ecosystem

HeraMind is a modular ecosystem with specialized resources for each concern:

| Resource | Purpose |
|------------|---------|
| **[HeraMind](https://github.com/CVEDIX/HeraMind)** | Core platform (this repo) — backend, frontend, desktop app |
| **[Extension docs](https://docs.cvedix.com/heramind)** | Official extension marketplace — weather, YOLO detection, OCR, face recognition, streaming |
| **[Device type docs](https://docs.cvedix.com/heramind)** | Device type definitions — standardized metrics and commands for IoT hardware |
| **[Dashboard component docs](https://docs.cvedix.com/heramind)** | Dashboard widget marketplace — community-contributed React components |

### Available Extensions

| Extension | Category | Description |
|-----------|----------|-------------|
| **Weather Forecast** | Data | Real-time weather via Open-Meteo API |
| **Image Analyzer** | Vision | YOLOv11 object detection on uploaded images (80+ COCO categories) |
| **YOLO Video** | Vision | Real-time object detection on RTSP/RTMP/HLS streams |
| **YOLO Device Inference** | Vision | Auto-detection on NE301/NE101 camera feeds |
| **Face Recognition** | Vision | ArcFace enrollment, matching, and real-time detection |
| **OCR Device Inference** | Vision | PP-OCRv4 text extraction from camera feeds |
| **Stream Player** | UI | RTSP/RTMP/HLS video player dashboard widget |
| **Home Assistant Bridge** | Integration | Bidirectional HA sync via REST + WebSocket |
| **LoRaWAN Bridge** | Integration | ChirpStack/TTN device data + payload decoding |
| **Modbus Bridge** | Integration | Modbus TCP/RTU register map decoding |
| **Uink-RMS Bridge** | Integration | E-paper display telemetry sync |

### Supported Devices

NE301 (Edge AI Camera) and NE101 (Sensing Camera). See the [device type docs](https://docs.cvedix.com/heramind) for full device type definitions.

### Contribute to the Ecosystem

We welcome community contributions to grow the HeraMind ecosystem:

- **[Build an Extension](https://docs.cvedix.com/heramind)** — Create extensions for new data sources, AI models, or integrations. Follow the [Extension Development Guide](https://docs.cvedix.com/heramind) to get started, then submit a PR to the marketplace.
- **[Add a Device Type](https://docs.cvedix.com/heramind)** — Define metrics and commands for your IoT hardware so others can use it out of the box. Just add a JSON file.
- **[Create a Dashboard Widget](https://docs.cvedix.com/heramind)** — Build reusable React dashboard components (charts, gauges, maps, etc.) and share them with the community.

## Quick Start

> For the full walkthrough see the [Five-Minute Guide](https://docs.cvedix.com/heramind) and [Install & Setup](https://docs.cvedix.com/heramind) in the docs.

### Desktop App (Recommended)

Download the latest release from [GitHub Releases](https://github.com/CVEDIX/HeraMind/releases/latest).

| Platform | Format |
|----------|--------|
| macOS (Apple Silicon + Intel) | `.dmg` |
| Windows | `.msi` / `.exe` |
| Linux | `.AppImage` / `.deb` |

On first launch, a setup wizard guides you through creating an admin account, configuring your LLM backend, and connecting devices.

### Server Deployment

One-line install (Linux & macOS):

```bash
curl -fsSL https://raw.githubusercontent.com/CVEDIX/HeraMind/main/scripts/install.sh | sh
```

Access the web UI at `http://your-server:9375`.

<details>
<summary>More installation options</summary>

**Docker:**

```bash
git clone https://github.com/CVEDIX/HeraMind.git
cd HeraMind
docker compose up -d
```

**Specific version:**
```bash
curl -fsSL https://raw.githubusercontent.com/CVEDIX/HeraMind/main/scripts/install.sh | VERSION=0.9.1 sh
```

**Custom directories:**
```bash
curl -fsSL ... | INSTALL_DIR=~/.local/bin DATA_DIR=~/.heramind sh
```

**Backend only (no web UI):**
```bash
curl -fsSL ... | NO_WEB=true sh
```

**With nginx reverse proxy (port 80):**
```bash
curl -fsSL ... | USE_NGINX=true sh
```

**Manual installation:**
```bash
VERSION=0.9.1
wget https://github.com/CVEDIX/HeraMind/releases/download/v${VERSION}/heramind-server-linux-amd64.tar.gz
wget https://github.com/CVEDIX/HeraMind/releases/download/v${VERSION}/heramind-web-${VERSION}.tar.gz
tar xzf heramind-server-linux-amd64.tar.gz
sudo install -m 755 heramind /usr/local/bin/
sudo install -m 755 heramind-extension-runner /usr/local/bin/
sudo mkdir -p /var/www/heramind
sudo tar xzf heramind-web-${VERSION}.tar.gz -C /var/www/heramind
./heramind serve
```

**Nginx config:**
```nginx
server {
    listen 80;
    root /var/www/heramind;
    index index.html;
    location / { try_files $uri $uri/ /index.html; }
    location /api/ {
        proxy_pass http://127.0.0.1:9375/api/;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

</details>

### Development

**Prerequisites:** Rust 1.85+, Node.js 20+, Ollama (or other LLM backend)

```bash
# Clone
git clone https://github.com/CVEDIX/HeraMind.git
cd HeraMind

# Start backend (port 9375)
cargo run -p heramind-cli -- serve

# Start frontend dev server (port 5173)
cd web && npm install && npm run dev

# Build desktop app
cd web && npm run tauri:build
```

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                  Desktop App / Web UI                         │
│                   React 18 + TypeScript                       │
├──────────────────────────────────────────────────────────────┤
│                   Tauri 2.x / Browser                         │
└────────────────────────┬─────────────────────────────────────┘
                         │ REST / WebSocket / SSE
                         ▼
┌──────────────────────────────────────────────────────────────┐
│                        API Gateway                            │
│                     Axum Web Server                           │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐    │
│  │ Auth   │ │Devices │ │Automate│ │Messages│ │Extension│   │
│  └────────┘ └────────┘ └────────┘ └────────┘ └────────┘    │
└────────────────────────┬─────────────────────────────────────┘
                         │ Event Bus
          ┌──────────────┼──────────────┬────────────────┐
          ▼              ▼              ▼                ▼
   ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐
   │ Devices  │  │Automation│  │ AI Agent │  │   Extensions     │
   │          │  │          │  │          │  │                  │
   │ MQTT     │  │ Rules    │  │ Chat     │  │ Process Isolated │
   │ BLE      │  │ Transform│  │ Tools    │  │ Native + WASM    │
   │ Webhook  │  │ Agents   │  │ Memory   │  │ Capabilities     │
   └──────────┘  └──────────┘  └──────────┘  └──────────────────┘
          │              │              │                │
          └──────────────┴──────────────┴────────────────┘
                         │
                         ▼
   ┌─────────────────────────────────────────────────────────┐
   │                    Storage Layer                          │
   │  ┌────────────┐ ┌────────────┐ ┌──────────┐ ┌────────┐ │
   │  │ Time-Series│ │   State    │ │   LLM    │ │  Push  │ │
   │  │  (redb)    │ │  (redb)    │ │  Memory  │ │  Logs  │ │
   │  └────────────┘ └────────────┘ └──────────┘ └────────┘ │
   └─────────────────────────────────────────────────────────┘
```

## Project Structure

```
HeraMind/
├── crates/
│   ├── heramind-core/            # Core traits and type system
│   ├── heramind-api/             # Web API server (Axum)
│   ├── heramind-agent/           # AI Agent, tool calling, LLM backends
│   ├── heramind-devices/         # Device management (MQTT, BLE, Webhook)
│   ├── heramind-storage/         # Storage layer (redb)
│   ├── heramind-messages/        # Notifications (7 channels)
│   ├── heramind-rules/           # Rule engine (JSON conditions/actions)
│   ├── heramind-data-push/       # Data push to external systems
│   ├── heramind-cli-ops/         # Shared CLI logic (in-process dispatch)
│   ├── heramind-extension-sdk/   # Extension development SDK
│   ├── heramind-extension-runner/# Extension process isolation
│   └── heramind-cli/             # Command-line interface
├── web/
│   ├── src/                     # React frontend (TypeScript)
│   └── src-tauri/               # Tauri desktop backend (Rust)
├── scripts/                     # Deployment scripts
├── docs/                        # Documentation
├── deploy/                      # Deployment configs (nginx, systemd)
├── Dockerfile                   # Multi-stage Docker build
├── docker-compose.yml           # Docker Compose configuration
└── .env.example                 # Environment variable template
```

## More Screenshots

<details>
<summary>Click to expand</summary>

<br/>

<table>
  <tr>
    <td><b>Login</b></td>
    <td><b>AI Chat</b></td>
  </tr>
  <tr>
    <td><img src="docs/img/login.png" width="480" /></td>
    <td><img src="docs/img/chat.png" width="480" /></td>
  </tr>
  <tr>
    <td><b>AI Agents</b></td>
    <td><b>Rules Engine</b></td>
  </tr>
  <tr>
    <td><img src="docs/img/agents.png" width="480" /></td>
    <td><img src="docs/img/rules.png" width="480" /></td>
  </tr>
  <tr>
    <td><b>Data Transforms</b></td>
    <td><b>Messages</b></td>
  </tr>
  <tr>
    <td><img src="docs/img/transforms.png" width="480" /></td>
    <td><img src="docs/img/messages.png" width="480" /></td>
  </tr>
  <tr>
    <td><b>Extensions</b></td>
    <td><b>Data Push</b></td>
  </tr>
  <tr>
    <td><img src="docs/img/extensions.png" width="480" /></td>
    <td><img src="docs/img/data-push.png" width="480" /></td>
  </tr>
  <tr>
    <td><b>LLM Backends</b></td>
    <td><b>Mobile</b></td>
  </tr>
  <tr>
    <td><img src="docs/img/llm-backends.png" width="480" /></td>
    <td><img src="docs/img/mobile_web.png" width="200" /></td>
  </tr>
</table>

</details>

## Configuration

### Supported LLM Backends

Ollama (local), OpenAI, Anthropic, Google, xAI, Qwen, DeepSeek, GLM, MiniMax, and any OpenAI-compatible endpoint. Configure via the **Settings → LLM Backends** page in the UI.

<details>
<summary>Environment variables</summary>

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level (trace, debug, info, warn, error) |
| `HERAMIND_DATA_DIR` | `/var/lib/heramind` | Data directory |
| `HERAMIND_BIND_ADDR` | `0.0.0.0:9375` | Server bind address |
| `SERVER_PORT` | `9375` | API server port |

</details>

## CLI Reference

```bash
heramind serve                          # Start API server
heramind health                        # System health check
heramind device list                   # List devices
heramind device create --name "..."    # Create device
heramind rule list                     # List automation rules
heramind extension list                # List extensions
heramind extension install file.nep    # Install extension
heramind agent list                    # List AI agents
heramind message list                  # List messages
heramind system info                   # System status & network info
heramind api-key create                # Create API key
```

## Extension Development

Build extensions using the Rust SDK with process isolation. See the [Developer Guide](https://docs.cvedix.com/heramind) and [HeraMind docs](https://docs.cvedix.com/heramind) for full examples.

<details>
<summary>Quick example</summary>

```rust
use heramind_extension_sdk::prelude::*;

pub struct MyExtension;

#[async_trait]
impl Extension for MyExtension {
    fn metadata(&self) -> &ExtensionMetadata {
        static META: OnceLock<ExtensionMetadata> = OnceLock::new();
        META.get_or_init(|| {
            ExtensionMetadata::new("my-extension", "My Extension", "1.0.0")
                .with_description("My custom extension")
                .with_author("Your Name")
        })
    }

    async fn execute_command(&self, cmd: &str, args: &Value) -> Result<Value> {
        match cmd {
            "do_something" => Ok(json!({ "result": "done" })),
            _ => Err(ExtensionError::CommandNotFound(cmd.to_string())),
        }
    }

    fn produce_metrics(&self) -> Result<Vec<ExtensionMetricValue>> {
        Ok(vec![])
    }
}

heramind_export!(MyExtension);
```

</details>

## Documentation

All user, install, and developer documentation lives on the **[HeraMind Docs](https://docs.cvedix.com/heramind)**:

| Docs Section | Covers |
|--------------|--------|
| [Product Overview](https://docs.cvedix.com/heramind) | What HeraMind is, core concepts, architecture |
| [Quick Start](https://docs.cvedix.com/heramind) | Five-minute guide to your first running instance |
| [Install & Setup](https://docs.cvedix.com/heramind) | Desktop app, server, Docker, configuration |
| [Developer Guide](https://docs.cvedix.com/heramind) | REST/WebSocket API, extensions, integrations |

Repo-local references (kept here for contributors):

| Resource | Description |
|----------|-------------|
| [CLAUDE.md](CLAUDE.md) | Development guide and code conventions |
| [CHANGELOG.md](CHANGELOG.md) | Version history and release notes |
| [Frontend Spec](web/DESIGN_SPEC.md) | UI design system and component standards |

## Tech Stack

| Layer | Technology |
|-------|-----------|
| **Backend** | Rust, Axum, Tokio, redb |
| **Frontend** | React 18, TypeScript, Tailwind CSS, Zustand, Radix UI |
| **Desktop** | Tauri 2.x |
| **AI/LLM** | Ollama, OpenAI, Anthropic, and 6+ more backends |
| **IoT** | MQTT (embedded broker), BLE, HTTP/Webhook |
| **Extensions** | Native (.so/.dylib/.dll), WASM, process isolation |

## Community

Join our community to get help, share ideas, and stay up to date:

- **[Discord](https://discord.gg/gkM7cc8gKb)** — Real-time chat, support, and announcements (recommended)
- **[GitHub Issues](https://github.com/CVEDIX/HeraMind/issues)** — Bug reports and feature requests
- **[GitHub Discussions](https://github.com/CVEDIX/HeraMind/discussions)** — Long-form Q&A and design talks
- **[HeraMind Docs](https://docs.cvedix.com/heramind)** — Full documentation

Release announcements are published to the Discord `#announcements` channel and on [GitHub Releases](https://github.com/CVEDIX/HeraMind/releases).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

[Apache-2.0](LICENSE)
