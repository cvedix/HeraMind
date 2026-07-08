#!/bin/bash
# macOS Signing Script for Open Source Projects
#
# Usage:
#   1. Without signing: ./scripts/sign-macos.sh --no-sign
#   2. With Apple ID:  ./scripts/sign-macos.sh --apple-id "your@email.com"
#   3. With developer certificate: ./scripts/sign-macos.sh --identity "Developer ID Application: Name"

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
BUNDLE_NAME="HeraMind.app"
DMG_NAME="HeraMind_0.1.0_aarch64.dmg"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Parse arguments
SIGN_MODE="none"
APPLE_ID=""
IDENTITY=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --no-sign)
            SIGN_MODE="none"
            shift
            ;;
        --apple-id)
            SIGN_MODE="apple-id"
            APPLE_ID="$2"
            shift 2
            ;;
        --identity)
            SIGN_MODE="developer"
            IDENTITY="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --no-sign           Build without signing (users need to bypass Gatekeeper)"
            echo "  --apple-id EMAIL    Sign with Apple ID (free, better UX)"
            echo "  --identity NAME     Sign with Developer ID (requires $99/year account)"
            echo "  --help              Show this help message"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

cd "$PROJECT_DIR/web/src-tauri"

# Check if build exists
if [ ! -d "target/release/bundle/dmg" ]; then
    log_info "Building application first..."
    cargo tauri build --bundles dmg
fi

DMG_PATH="target/release/bundle/dmg/$DMG_NAME"

if [ ! -f "$DMG_PATH" ]; then
    log_error "DMG not found at: $DMG_PATH"
    exit 1
fi

case $SIGN_MODE in
    none)
        log_info "Building without signing..."
        log_warn "Users will need to: Right-click -> Open, or use 'xattr -d'"
        ;;

    apple-id)
        log_info "Signing with Apple ID: $APPLE_ID"

        # Sign the app bundle inside the DMG
        log_info "Mounting DMG..."
        MOUNT_DIR=$(hdiutil attach "$DMG_PATH" -readonly -mountpoint /tmp/heramind-dmg | grep "/Volumes" | awk '{print $3}')

        log_info "Signing application bundle..."
        codesign --force --deep --sign "$APPLE_ID" "/tmp/heramind-dmg/HeraMind.app"

        log_info "Verifying signature..."
        codesign --verify --verbose "/tmp/heramind-dmg/HeraMind.app" || true

        log_info "Unmounting DMG..."
        hdiutil detach "/tmp/heramind-dmg" || true

        log_info "Re-creating signed DMG..."
        TMP_DMG="HeraMind_signed.dmg"
        hdiutil create -volname "HeraMind" -srcfolder "/tmp/heramind-dmg/HeraMind.app" -ov -format UDZO "$TMP_DMG"

        mv "$TMP_DMG" "$DMG_PATH"
        rm -rf "$TMP_DMG"

        log_info "✅ Signed with Apple ID: $APPLE_ID"
        ;;

    developer)
        log_info "Signing with Developer Identity: $IDENTITY"

        # Find available signing identities
        log_info "Available signing identities:"
        security find-identity -v -p codesigning | grep -E "Developer ID Application|Apple Development"

        # Sign the app bundle
        log_info "Signing application bundle..."
        codesign --force --deep --sign "$IDENTITY" "target/release/bundle/macos/HeraMind.app"

        log_info "Verifying signature..."
        codesign --verify --verbose "target/release/bundle/macos/HeraMind.app"

        # Re-create DMG with signed app
        log_info "Re-creating signed DMG..."
        rm -f "$DMG_PATH"
        cargo tauri build --bundles dmg

        log_info "✅ Signed with Developer Identity"
        ;;
esac

log_info "Build complete: $DMG_PATH"
log_info ""
log_info "For GitHub Releases, upload:"
log_info "  - $DMG_PATH"
log_info ""
