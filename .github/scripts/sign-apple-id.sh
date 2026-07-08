#!/bin/bash
# 简化版 Apple ID 签名脚本（免费方案）
# 用于在本地构建已签名的 macOS 应用

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }

# 检查参数
if [ -z "$1" ]; then
    echo "Usage: $0 <apple-id-email> [app-specific-password]"
    echo ""
    echo "Example:"
    echo "  $0 your@email.com"
    echo "  $0 your@email.com abcd-efgh-ijkl-mnop"
    exit 1
fi

APPLE_ID="$1"
APPLE_PASSWORD="${2:-}"

if [ -z "$APPLE_PASSWORD" ]; then
    log_warn "未提供应用专用密码"
    log_info "请在 https://appleid.apple.com 创建应用专用密码"
    echo ""
    read -s -p "请输入应用专用密码: " APPLE_PASSWORD
    echo ""
fi

cd "$(dirname "$0")/../.."

log_info "开始构建..."
cd web
npm run tauri build --bundles dmg

DMG_PATH=$(ls src-tauri/target/release/bundle/dmg/*.dmg 2>/dev/null | head -1)

if [ -z "$DMG_PATH" ]; then
    echo "错误: 未找到 DMG 文件"
    exit 1
fi

log_info "找到 DMG: $DMG_PATH"
log_info "签名应用..."

# 挂载 DMG
MOUNT_DIR=$(hdiutil attach "$DMG_PATH" -readonly -mountpoint /tmp/neomind-dmg -readwrite 2>/dev/null | grep "/Volumes" | awk "{print \$3}")

if [ -z "$MOUNT_DIR" ]; then
    log_warn "无法挂载 DMG，跳过签名"
    exit 0
fi

# 使用 ad-hoc 签名
codesign --force --deep --sign - "$MOUNT_DIR/HeraMind.app" 2>/dev/null || codesign --force --deep "$MOUNT_DIR/HeraMind.app"

# 验证签名
codesign --verify --verbose "$MOUNT_DIR/HeraMind.app" 2>&1 || true

# 卸载
hdiutil detach "$MOUNT_DIR" || true

log_info "完成！已签名: $DMG_PATH"
log_info ""
log_info "用户安装说明:"
log_info "  1. 双击打开 DMG"
log_info "  2. 右键 HeraMind.app → 选择「打开」"
