<p align="center">
  <img src="web/public/logo-light.png" alt="HeraMind" width="400">
</p>

<h3 align="center">面向物联网自动化的边缘 AI 平台</h3>

<p align="center">
  基于 Rust 的边缘智能 — 连接设备、运行 AI 智能体、自动化一切。
</p>

<p align="center">
  <a href="https://github.com/camthink-ai/HeraMind/actions/workflows/build.yml">
    <img src="https://github.com/camthink-ai/HeraMind/actions/workflows/build.yml/badge.svg" alt="构建状态">
  </a>
  <img src="https://img.shields.io/badge/License-Apache--2.0-blue.svg" alt="许可证">
  <img src="https://img.shields.io/badge/v-0.8.11-information.svg" alt="版本">
  <img src="https://img.shields.io/badge/Rust-1.85+-orange.svg" alt="Rust">
  <img src="https://img.shields.io/badge/平台-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey.svg" alt="平台支持">
</p>

<br/>

<div align="center">
  <img src="https://resources.camthink.ai/HeraMind/dashboardDemo.png" alt="仪表板" width="800" style="border-radius: 8px; box-shadow: 0 4px 16px rgba(0,0,0,0.12);" />
  <br/><sub><b>仪表板</b></sub>
</div>

<br/>

<div align="center">
  <table>
    <tr>
      <td align="center">
        <img src="docs/img/chat.png" alt="AI 对话" width="400" style="border-radius: 8px; box-shadow: 0 4px 16px rgba(0,0,0,0.12);" />
        <br/><sub><b>AI 对话</b></sub>
      </td>
      <td align="center">
        <img src="docs/img/devices.png" alt="设备" width="400" style="border-radius: 8px; box-shadow: 0 4px 16px rgba(0,0,0,0.12);" />
        <br/><sub><b>设备管理</b></sub>
      </td>
      <td align="center">
        <img src="docs/img/mobile_web.png" alt="移动端" width="180" style="border-radius: 8px; box-shadow: 0 4px 16px rgba(0,0,0,0.12);" />
        <br/><sub><b>移动端</b></sub>
      </td>
    </tr>
  </table>
</div>

<br/>

## HeraMind 是什么？

HeraMind 是一个**边缘部署的 AI 平台**，将智能带入物联网。它在本地硬件上直接运行 LLM 驱动的智能体，通过 MQTT/BLE/Webhook 连接设备，利用规则引擎实现自动化响应，并在实时仪表板上可视化一切 — 无需依赖云服务。

**核心理念**：用自然语言与你的设备对话。AI 理解你的意图，查询设备状态，创建自动化规则，并自主执行操作。

> 📚 **完整文档已迁移至 [HeraMind Wiki](https://wiki.camthink.ai/zh-Hans/docs/heramind/product-overview/what-is-heramind)。** 本 README 仅作快速概览，详细指南请访问 Wiki：
>
> - [HeraMind 是什么？](https://wiki.camthink.ai/zh-Hans/docs/heramind/product-overview/what-is-heramind) — 产品概览与核心理念
> - [五分钟快速开始](https://wiki.camthink.ai/zh-Hans/docs/heramind/quick-start/five-minute-guide) — 快速上手
> - [安装与配置](https://wiki.camthink.ai/zh-Hans/docs/heramind/user-guide/install-setup) — 桌面端、服务器、Docker 部署
> - [开发者指南](https://wiki.camthink.ai/zh-Hans/docs/heramind/developer-guide/overview) — API、扩展、集成

## 核心特性

### AI 智能
- **自然语言对话** — 用自然语言查询和控制所有已连接设备
- **自主智能体** — 定时 AI 智能体，独立监控、分析和处理设备数据
- **10+ LLM 后端** — Ollama、OpenAI、Anthropic、Google、xAI、Qwen、DeepSeek、GLM、MiniMax 及任何 OpenAI 兼容接口
- **记忆系统** — 多层级记忆（用户画像、知识库、任务记录、系统演化），支持自动提取和压缩
- **技能系统** — YAML+Markdown 技能，引导智能体在特定场景下的行为
- **多模态** — 支持图片上传和视觉分析

### 设备管理
- **MQTT 协议** — 主要设备集成方式，支持嵌入式 Broker、mTLS 和 CA 证书
- **BLE 配网** — 通过蓝牙零接触设备设置（Tauri 原生 + Web Bluetooth）
- **HTTP/Webhook** — 灵活的 REST 设备适配器
- **自动发现** — 自动检测设备、注册类型，AI 辅助设备入板
- **命令队列** — 向设备发送控制命令，支持参数验证和执行跟踪
- **自定义设备类型** — 通过 JSON 定义设备指标和命令

### 自动化
- **DSL 规则引擎** — 人类可读的规则语言：`WHEN device("sensor").temperature > 30 DO device("ac").power_on()`
- **数据转换** — 基于 JavaScript 的数据转换，创建虚拟指标
- **定时智能体** — 基于时间和事件驱动的 AI 智能体执行
- **事件总线** — 发布/订阅架构，组件间解耦通信

### 仪表板与可视化
- **拖拽式构建器** — 可视化仪表板编辑器，响应式网格布局
- **丰富组件** — 数值卡片、图表、仪表盘、表格、VLM 视觉组件
- **实时更新** — WebSocket/SSE 实时数据推送到仪表板
- **仪表板分享** — 带过期时间的公开链接
- **自定义组件** — 构建并发布你自己的仪表板组件

### 通知与数据推送
- **7 种通知渠道** — Webhook、邮件、Telegram、企业微信、钉钉、Slack、飞书
- **数据推送** — 通过 Webhook 或 MQTT 将遥测数据转发到外部系统
- **投递追踪** — 指数退避重试逻辑、投递历史和日志管理
- **消息去重** — 防止高频触发造成的通知风暴

### 平台能力
- **多实例管理** — 从单一界面连接和管理多个 HeraMind 后端
- **扩展系统** — Native 和 WASM 扩展，进程隔离和基于能力的权限控制
- **跨平台桌面** — 通过 Tauri 提供 macOS、Windows、Linux 原生应用
- **移动端适配** — 针对手机和平板优化的响应式 Web 界面
- **国际化** — 支持中文和英文
- **深色模式** — 系统自适应的深色/浅色主题
- **API 密钥认证** — JWT 的替代方案，适合程序化访问
- **CLI 工具** — 全功能命令行界面

## 生态系统

HeraMind 是一个模块化的生态系统，每个关注点都有专门的仓库：

| 仓库 | 用途 |
|------|------|
| **[HeraMind](https://github.com/camthink-ai/HeraMind)** | 核心平台（本仓库）— 后端、前端、桌面应用 |
| **[HeraMind-Extensions](https://github.com/camthink-ai/HeraMind-Extensions)** | 官方扩展市场 — 天气、YOLO 检测、OCR、人脸识别、视频流 |
| **[HeraMind-DeviceTypes](https://github.com/camthink-ai/HeraMind-DeviceTypes)** | 设备类型定义 — 标准化的 IoT 硬件指标和命令 |
| **[HeraMind-Dashboard-Components](https://github.com/camthink-ai/HeraMind-Dashboard-Components)** | 仪表板组件市场 — 社区贡献的 React 组件 |

### 可用扩展

| 扩展 | 说明 |
|------|------|
| **天气预报** | 通过 Open-Meteo API 获取实时天气数据。提供温度、湿度、风速和降水量等指标，可作为仪表板和自动化规则的数据源。支持可配置的地理位置和轮询间隔。 |
| **图像分析** | 基于 YOLOv11 的上传图片目标检测。可识别人、车辆、动物等 80+ 种 COCO 类别，返回边界框、置信度和类别标签等结构化数据。 |
| **YOLO 视频** | 实时视频流（RTSP/RTMP/HLS）目标检测。可配置帧率处理，采用丢弃中间帧队列实现低延迟。支持叠加渲染和检测计数指标。 |
| **YOLO 设备推理** | 自动对设备摄像头画面运行 YOLO 检测。绑定 NE301/NE101 摄像头流，将检测结果发布为设备指标。支持检测到特定目标时触发 AI 告警。 |
| **人脸识别** | 基于 ArcFace 的人脸识别，支持注册和匹配。支持人脸库管理、摄像头实时检测，以及基于置信度阈值的门禁场景匹配。 |
| **OCR 设备推理** | 基于 PP-OCRv4 的设备摄像头文字识别。从图片和视频帧中提取文本，支持多语言识别。适用于仪表读数、车牌识别和文档处理。 |
| **流媒体播放器** | 仪表板视频播放组件，支持 RTSP、RTMP 和 HLS 协议。提供低延迟播放、截图捕获、全屏模式和设备指标叠加显示。 |

### 支持设备

NE301（边缘 AI 摄像头）和 NE101（感知摄像头）。完整设备类型定义请查看 [HeraMind-DeviceTypes](https://github.com/camthink-ai/HeraMind-DeviceTypes)。

### 参与生态共建

我们欢迎社区贡献，共同丰富 HeraMind 生态系统：

- **[开发扩展](https://github.com/camthink-ai/HeraMind-Extensions)** — 为新的数据源、AI 模型或集成创建扩展。参考 [开发者指南](https://wiki.camthink.ai/zh-Hans/docs/heramind/developer-guide/overview) 快速上手，然后向市场提交 PR。
- **[添加设备类型](https://github.com/camthink-ai/HeraMind-DeviceTypes)** — 为你的 IoT 硬件定义指标和命令，让其他人开箱即用。只需添加一个 JSON 文件。
- **[创建仪表板组件](https://github.com/camthink-ai/HeraMind-Dashboard-Components)** — 构建可复用的 React 仪表板组件（图表、仪表盘、地图等），与社区分享。

## 快速开始

> 完整流程请参考 Wiki 的 [五分钟快速开始](https://wiki.camthink.ai/zh-Hans/docs/heramind/quick-start/five-minute-guide) 和 [安装与配置](https://wiki.camthink.ai/zh-Hans/docs/heramind/user-guide/install-setup)。

### 桌面应用（推荐）

从 [GitHub Releases](https://github.com/camthink-ai/HeraMind/releases/latest) 下载最新版本。

| 平台 | 格式 |
|------|------|
| macOS (Apple Silicon + Intel) | `.dmg` |
| Windows | `.msi` / `.exe` |
| Linux | `.AppImage` / `.deb` |

首次启动时，设置向导会引导你创建管理员账户、配置 LLM 后端、连接设备。

### 服务器部署

一键安装（Linux & macOS）：

```bash
curl -fsSL https://raw.githubusercontent.com/camthink-ai/HeraMind/main/scripts/install.sh | sh
```

安装后访问 `http://your-server:9375`。

<details>
<summary>更多安装选项</summary>

**Docker 部署：**

```bash
git clone https://github.com/camthink-ai/HeraMind.git
cd HeraMind
docker compose up -d
```

**指定版本：**
```bash
curl -fsSL https://raw.githubusercontent.com/camthink-ai/HeraMind/main/scripts/install.sh | VERSION=0.8.11 sh
```

**自定义目录：**
```bash
curl -fsSL ... | INSTALL_DIR=~/.local/bin DATA_DIR=~/.heramind sh
```

**仅后端（无 Web UI）：**
```bash
curl -fsSL ... | NO_WEB=true sh
```

**使用 nginx 反向代理（端口 80）：**
```bash
curl -fsSL ... | USE_NGINX=true sh
```

**手动安装：**
```bash
VERSION=0.8.11
wget https://github.com/camthink-ai/HeraMind/releases/download/v${VERSION}/heramind-server-linux-amd64.tar.gz
wget https://github.com/camthink-ai/HeraMind/releases/download/v${VERSION}/heramind-web-${VERSION}.tar.gz
tar xzf heramind-server-linux-amd64.tar.gz
sudo install -m 755 heramind /usr/local/bin/
sudo install -m 755 heramind-extension-runner /usr/local/bin/
sudo mkdir -p /var/www/heramind
sudo tar xzf heramind-web-${VERSION}.tar.gz -C /var/www/heramind
./heramind serve
```

**Nginx 配置：**
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

### 开发模式

**环境要求：** Rust 1.85+、Node.js 20+、Ollama（或其他 LLM 后端）

```bash
# 克隆
git clone https://github.com/camthink-ai/HeraMind.git
cd HeraMind

# 启动后端（端口 9375）
cargo run -p heramind-cli -- serve

# 启动前端开发服务器（端口 5173）
cd web && npm install && npm run dev

# 构建桌面应用
cd web && npm run tauri:build
```

## 系统架构

```raw
┌──────────────────────────────────────────────────────────────┐
│                    桌面应用 / Web 界面                         │
│                    React 18 + TypeScript                      │
├──────────────────────────────────────────────────────────────┤
│                    Tauri 2.x / 浏览器                         │
└────────────────────────┬─────────────────────────────────────┘
                         │ REST / WebSocket / SSE
                         ▼
┌──────────────────────────────────────────────────────────────┐
│                         API 网关                              │
│                      Axum Web 服务器                          │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐    │
│  │ 认证   │ │ 设备   │ │ 自动化 │ │ 消息   │ │ 扩展   │    │
│  └────────┘ └────────┘ └────────┘ └────────┘ └────────┘    │
└────────────────────────┬─────────────────────────────────────┘
                         │ 事件总线
          ┌──────────────┼──────────────┬────────────────┐
          ▼              ▼              ▼                ▼
   ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐
   │ 设备管理 │  │  自动化  │  │ AI 智能体│  │     扩展系统     │
   │          │  │          │  │          │  │                  │
   │ MQTT     │  │ 规则     │  │ 对话     │  │ 进程隔离         │
   │ BLE      │  │ 数据转换 │  │ 工具调用 │  │ Native + WASM   │
   │ Webhook  │  │ 智能体   │  │ 记忆系统 │  │ 能力权限         │
   └──────────┘  └──────────┘  └──────────┘  └──────────────────┘
          │              │              │                │
          └──────────────┴──────────────┴────────────────┘
                         │
                         ▼
   ┌─────────────────────────────────────────────────────────┐
   │                       存储层                             │
   │  ┌────────────┐ ┌────────────┐ ┌──────────┐ ┌────────┐ │
   │  │  时序数据  │ │   状态     │ │ LLM 记忆 │ │ 推送   │ │
   │  │  (redb)    │ │  (redb)    │ │          │ │ 日志   │ │
   │  └────────────┘ └────────────┘ └──────────┘ └────────┘ │
   └─────────────────────────────────────────────────────────┘
```

## 项目结构

```raw
HeraMind/
├── crates/
│   ├── heramind-core/            # 核心 traits 和类型系统
│   ├── heramind-api/             # Web API 服务器（Axum）
│   ├── heramind-agent/           # AI 智能体、工具调用、LLM 后端
│   ├── heramind-devices/         # 设备管理（MQTT、BLE、Webhook）
│   ├── heramind-storage/         # 存储层（redb）
│   ├── heramind-messages/        # 通知系统（7 种渠道）
│   ├── heramind-rules/           # DSL 规则引擎
│   ├── heramind-data-push/       # 数据推送到外部系统
│   ├── heramind-extension-sdk/   # 扩展开发 SDK
│   ├── heramind-extension-runner/# 扩展进程隔离运行器
│   └── heramind-cli/             # 命令行工具
├── web/
│   ├── src/                     # React 前端（TypeScript）
│   └── src-tauri/               # Tauri 桌面后端（Rust）
├── scripts/                     # 部署脚本
├── docs/                        # 文档
├── deploy/                      # 部署配置（nginx、systemd）
├── Dockerfile                   # 多阶段 Docker 构建
├── docker-compose.yml           # Docker Compose 配置
└── .env.example                 # 环境变量模板
```

## 更多截图

<details>
<summary>点击展开</summary>

<br/>

<table>
  <tr>
    <td><b>登录</b></td>
    <td><b>AI 对话</b></td>
  </tr>
  <tr>
    <td><img src="docs/img/login.png" width="480" /></td>
    <td><img src="docs/img/chat.png" width="480" /></td>
  </tr>
  <tr>
    <td><b>AI 智能体</b></td>
    <td><b>规则引擎</b></td>
  </tr>
  <tr>
    <td><img src="docs/img/agents.png" width="480" /></td>
    <td><img src="docs/img/rules.png" width="480" /></td>
  </tr>
  <tr>
    <td><b>数据转换</b></td>
    <td><b>消息通知</b></td>
  </tr>
  <tr>
    <td><img src="docs/img/transforms.png" width="480" /></td>
    <td><img src="docs/img/messages.png" width="480" /></td>
  </tr>
  <tr>
    <td><b>扩展系统</b></td>
    <td><b>数据推送</b></td>
  </tr>
  <tr>
    <td><img src="docs/img/extensions.png" width="480" /></td>
    <td><img src="docs/img/data-push.png" width="480" /></td>
  </tr>
  <tr>
    <td><b>LLM 后端</b></td>
    <td><b>移动端</b></td>
  </tr>
  <tr>
    <td><img src="docs/img/llm-backends.png" width="480" /></td>
    <td><img src="docs/img/mobile_web.png" width="200" /></td>
  </tr>
</table>

</details>

## 配置

### LLM 后端

| 后端 | 端点 |
|------|------|
| Ollama | `http://localhost:11434` |
| OpenAI | `https://api.openai.com/v1` |
| Anthropic | `https://api.anthropic.com/v1` |
| Google | `https://generativelanguage.googleapis.com/v1beta` |
| xAI | `https://api.x.ai/v1` |
| Qwen | `https://dashscope.aliyuncs.com/compatible-mode/v1` |
| DeepSeek | `https://api.deepseek.com/v1` |
| GLM | `https://open.bigmodel.cn/api/paas/v4` |
| MiniMax | `https://api.minimax.chat/v1` |

### 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `RUST_LOG` | `info` | 日志级别（trace、debug、info、warn、error） |
| `HERAMIND_DATA_DIR` | `/var/lib/heramind` | 数据目录 |
| `HERAMIND_BIND_ADDR` | `0.0.0.0:9375` | 服务器绑定地址 |
| `SERVER_PORT` | `9375` | API 服务器端口 |

## CLI 参考

```bash
heramind serve                          # 启动 API 服务器
heramind health                        # 系统健康检查
heramind device list                   # 列出设备
heramind device create --name "..."    # 创建设备
heramind rule list                     # 列出自动化规则
heramind extension list                # 列出扩展
heramind extension install file.nep    # 安装扩展
heramind agent list                    # 列出 AI 智能体
heramind message list                  # 列出消息
heramind system info                   # 系统状态和网络信息
heramind api-key create                # 创建 API 密钥
```

## 扩展开发

使用 Rust SDK 构建进程隔离的扩展：

```rust
use heramind_extension_sdk::prelude::*;

pub struct MyExtension;

#[async_trait]
impl Extension for MyExtension {
    fn metadata(&self) -> &ExtensionMetadata {
        static META: OnceLock<ExtensionMetadata> = OnceLock::new();
        META.get_or_init(|| {
            ExtensionMetadata::new("my-extension", "我的扩展", "1.0.0")
                .with_description("我的自定义扩展")
                .with_author("你的名字")
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

详见 [开发者指南](https://wiki.camthink.ai/zh-Hans/docs/heramind/developer-guide/overview) 和 [HeraMind-Extensions](https://github.com/camthink-ai/HeraMind-Extensions)。

## 文档

所有用户、安装和开发文档均位于 **[HeraMind Wiki](https://wiki.camthink.ai/zh-Hans/docs/heramind/product-overview/what-is-heramind)**：

| Wiki 板块 | 内容 |
|----------|------|
| [产品概览](https://wiki.camthink.ai/zh-Hans/docs/heramind/product-overview/what-is-heramind) | HeraMind 是什么、核心概念、架构 |
| [快速开始](https://wiki.camthink.ai/zh-Hans/docs/heramind/quick-start/five-minute-guide) | 五分钟跑起第一个实例 |
| [安装与配置](https://wiki.camthink.ai/zh-Hans/docs/heramind/user-guide/install-setup) | 桌面端、服务器、Docker、配置 |
| [开发者指南](https://wiki.camthink.ai/zh-Hans/docs/heramind/developer-guide/overview) | REST/WebSocket API、扩展、集成 |

仓库内保留的本地参考（面向贡献者）：

| 资源 | 说明 |
|------|------|
| [CLAUDE.md](CLAUDE.md) | 开发指南和代码规范 |
| [CHANGELOG.md](CHANGELOG.md) | 版本历史和发布说明 |
| [前端规范](web/DESIGN_SPEC.md) | UI 设计系统和组件标准 |

## 技术栈

| 层级 | 技术 |
|------|------|
| **后端** | Rust、Axum、Tokio、redb |
| **前端** | React 18、TypeScript、Tailwind CSS、Zustand、Radix UI |
| **桌面** | Tauri 2.x |
| **AI/LLM** | Ollama、OpenAI、Anthropic 及 6+ 其他后端 |
| **IoT** | MQTT（嵌入式 Broker）、BLE、HTTP/Webhook |
| **扩展** | Native（.so/.dylib/.dll）、WASM、进程隔离 |

## 贡献

欢迎贡献！请随时提交 Pull Request。

## 许可证

[Apache-2.0](LICENSE)
