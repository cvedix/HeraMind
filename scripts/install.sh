#!/bin/sh
# HeraMind Installation Script
# Usage: curl -fsSL https://raw.githubusercontent.com/camthink-ai/HeraMind/main/scripts/install.sh | sh
#
# Environment variables:
#   VERSION        - Specific version to install (default: latest)
#   INSTALL_DIR    - Installation directory (default: /usr/local/bin)
#   DATA_DIR       - Data directory (default: /var/lib/heramind)
#   WEB_DIR        - Frontend static files directory (default: /var/www/heramind)
#   NO_WEB        - Skip frontend installation, backend only (default: false)
#   NO_SERVICE     - Skip service installation (default: false)
#   USE_NGINX      - Configure nginx reverse proxy (default: false)
#   PORT           - Backend API port (default: 9375)

set -eu

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

# Configuration
REPO="camthink-ai/HeraMind"
VERSION="${VERSION:-}"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
DATA_DIR="${DATA_DIR:-/var/lib/heramind}"
WEB_DIR="${WEB_DIR:-/var/www/heramind}"
NO_WEB="${NO_WEB:-false}"
NO_SERVICE="${NO_SERVICE:-false}"
USE_NGINX="${USE_NGINX:-false}"
PORT="${PORT:-9375}"

status() { echo "${BLUE}[INFO]${NC} $*"; }
success() { echo "${GREEN}[OK]${NC} $*"; }
warning() { echo "${YELLOW}[WARN]${NC} $*"; }
error() { echo "${RED}[ERROR]${NC} $*" >&2; exit 1; }

cleanup() {
    if [ -n "${TEMP_DIR:-}" ] && [ -d "$TEMP_DIR" ]; then
        rm -rf "$TEMP_DIR"
    fi
}
trap cleanup EXIT

available() { command -v "$1" >/dev/null 2>&1; }

require() {
    local MISSING=''
    for TOOL in "$@"; do
        if ! available "$TOOL"; then
            MISSING="$MISSING $TOOL"
        fi
    done
    if [ -n "$MISSING" ]; then
        error "Missing required tools:$MISSING. Please install them first."
    fi
}

get_os() {
    OS=$(uname -s)
    case "$OS" in
        Darwin) OS="darwin" ;;
        Linux) OS="linux" ;;
        *) error "Unsupported OS: $OS" ;;
    esac
}

get_arch() {
    ARCH=$(uname -m)
    case "$ARCH" in
        x86_64|amd64) ARCH="amd64" ;;
        aarch64|arm64) ARCH="arm64" ;;
        *) error "Unsupported architecture: $ARCH" ;;
    esac
}

get_latest_version() {
    status "Fetching latest version..."
    VERSION=$(curl -sfL https://api.github.com/repos/${REPO}/releases/latest |
              grep '"tag_name":' | sed -E 's/.*"v([^"]+)".*/\1/')
    if [ -z "$VERSION" ]; then
        error "Failed to fetch latest version from GitHub"
    fi
}

detect_sudo() {
    if [ "$(id -u)" -ne 0 ]; then
        if available sudo; then
            SUDO="sudo"
        else
            error "This script requires root privileges. Please run with sudo or as root."
        fi
    else
        SUDO=""
    fi
}

install_linux() {
    status "Installing HeraMind on Linux..."

    # Create user if not exists
    if ! id -u heramind >/dev/null 2>&1; then
        status "Creating heramind user..."
        $SUDO useradd -r -s /bin/false -d "$DATA_DIR" heramind 2>/dev/null || true
    fi

    # Create directories
    status "Creating directories..."
    $SUDO mkdir -p "$INSTALL_DIR"
    $SUDO mkdir -p "$DATA_DIR"
    $SUDO chown -R heramind:heramind "$DATA_DIR"

    # Download and extract server binaries
    BINARY_FILE="heramind-server-linux-${ARCH}.tar.gz"
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/v${VERSION}/${BINARY_FILE}"

    status "Downloading HeraMind server v${VERSION} for ${OS}/${ARCH}..."
    TEMP_DIR=$(mktemp -d)

    if ! curl -fSL --progress-bar "$DOWNLOAD_URL" -o "$TEMP_DIR/heramind.tar.gz"; then
        error "Failed to download from $DOWNLOAD_URL"
    fi

    status "Extracting server..."
    tar xzf "$TEMP_DIR/heramind.tar.gz" -C "$TEMP_DIR"

    # Install binary
    status "Installing binary to $INSTALL_DIR..."
    $SUDO install -m 755 "$TEMP_DIR/heramind" "$INSTALL_DIR/heramind"

    # Install extension runner if present
    if [ -f "$TEMP_DIR/heramind-extension-runner" ]; then
        $SUDO install -m 755 "$TEMP_DIR/heramind-extension-runner" "$INSTALL_DIR/heramind-extension-runner"
        success "Extension runner installed"
    fi

    # Download and extract frontend
    if [ "$NO_WEB" = "true" ]; then
        status "Skipping frontend (NO_WEB=true). Backend-only deployment."
    else
        WEB_FILE="heramind-web-${VERSION}.tar.gz"
    WEB_URL="https://github.com/${REPO}/releases/download/v${VERSION}/${WEB_FILE}"

    status "Downloading frontend..."
    if curl -fSL --progress-bar "$WEB_URL" -o "$TEMP_DIR/heramind-web.tar.gz" 2>/dev/null; then
        # Extract to a staging dir, then atomically swap into place. This
        # avoids accumulating stale hashed assets from previous versions
        # (Vite emits content-hashed filenames, so old files never get
        # overwritten and would pile up across upgrades).
        WEB_NEW="${WEB_DIR}.new.$$"
        WEB_OLD="${WEB_DIR}.old.$$"
        $SUDO rm -rf "$WEB_NEW" "$WEB_OLD"
        $SUDO mkdir -p "$WEB_NEW"
        $SUDO tar xzf "$TEMP_DIR/heramind-web.tar.gz" -C "$WEB_NEW"
        $SUDO chown -R www-data:www-data "$WEB_NEW" 2>/dev/null || \
            $SUDO chown -R heramind:heramind "$WEB_NEW"
        if [ -d "$WEB_DIR" ]; then
            $SUDO mv "$WEB_DIR" "$WEB_OLD"
        fi
        $SUDO mv "$WEB_NEW" "$WEB_DIR"
        $SUDO rm -rf "$WEB_OLD"
        success "Frontend installed to $WEB_DIR"
    else
        warning "Frontend package not found. Web UI will show a placeholder page."
        warning "You can manually download it from the release page."
        fi
    fi

    # Stop existing service before upgrading
    if $SUDO systemctl is-active --quiet heramind 2>/dev/null; then
        status "Stopping existing HeraMind service..."
        $SUDO systemctl stop heramind || true
    fi

    # Install systemd service
    if [ "$NO_SERVICE" != "true" ]; then
        status "Installing systemd service..."
        $SUDO tee /etc/systemd/system/heramind.service >/dev/null <<EOF
[Unit]
Description=HeraMind Edge AI Platform
Documentation=https://github.com/camthink-ai/HeraMind
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=heramind
Group=heramind
WorkingDirectory=${DATA_DIR}
ExecStart=${INSTALL_DIR}/heramind serve --host 0.0.0.0 --port ${PORT}
Restart=always
RestartSec=3
TimeoutStopSec=30

# Environment
Environment=RUST_LOG=info
Environment=HERAMIND_WEB_DIR=${WEB_DIR}
Environment=HERAMIND_API_BASE=http://localhost:${PORT}/api

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=full
ProtectHome=read-only
ReadWritePaths=${DATA_DIR} ${WEB_DIR}

[Install]
WantedBy=multi-user.target
EOF
        $SUDO systemctl daemon-reload
        $SUDO systemctl enable heramind
        success "Systemd service installed"
    fi

    # Configure nginx (optional, for frontend-backend separation)
    if [ "$USE_NGINX" = "true" ]; then
        if available nginx; then
            status "Configuring nginx..."
            $SUDO tee /etc/nginx/sites-available/heramind >/dev/null <<'EOF'
server {
    listen 80;
    server_name _;

    # Frontend static files
    root WEB_DIR_PLACEHOLDER;
    index index.html;

    # Gzip compression
    gzip on;
    gzip_types text/plain text/css application/json application/javascript text/xml application/xml text/javascript image/svg+xml;
    gzip_min_length 256;

    # SPA routing - serve index.html for all non-file routes
    location / {
        try_files $uri $uri/ /index.html;
    }

    # API reverse proxy
    location /api/ {
        proxy_pass http://127.0.0.1:PORT_PLACEHOLDER/api/;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 86400;
        proxy_send_timeout 86400;
    }

    # WebSocket reverse proxy
    location ~ ^/api/.*/ws$ {
        proxy_pass http://127.0.0.1:PORT_PLACEHOLDER;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_read_timeout 86400;
    }

    # SSE reverse proxy
    location /api/events/ {
        proxy_pass http://127.0.0.1:PORT_PLACEHOLDER/api/events/;
        proxy_http_version 1.1;
        proxy_set_header Connection '';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_buffering off;
        proxy_cache off;
        proxy_read_timeout 86400;
    }

    # Static asset caching
    location /assets/ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
EOF
            # Replace placeholders with actual values
            $SUDO sed -i "s|WEB_DIR_PLACEHOLDER|${WEB_DIR}|g" /etc/nginx/sites-available/heramind
            $SUDO sed -i "s|PORT_PLACEHOLDER|${PORT}|g" /etc/nginx/sites-available/heramind

            # Enable site
            if [ ! -L /etc/nginx/sites-enabled/heramind ]; then
                $SUDO ln -sf /etc/nginx/sites-available/heramind /etc/nginx/sites-enabled/heramind
            fi

            # Remove default site if it exists and heramind is the only site
            if [ -L /etc/nginx/sites-enabled/default ]; then
                $SUDO rm -f /etc/nginx/sites-enabled/default
            fi

            # Test and reload nginx
            if $SUDO nginx -t 2>/dev/null; then
                $SUDO systemctl reload nginx 2>/dev/null || $SUDO systemctl restart nginx 2>/dev/null || true
                success "Nginx configured and reloaded"
            else
                warning "Nginx config test failed. Please check /etc/nginx/sites-available/heramind"
            fi
        else
            warning "nginx not found. Skipping nginx configuration."
            warning "The server will serve frontend directly on port ${PORT}."
        fi
    fi

    # Configure firewall
    status "Configuring firewall..."
    if available ufw; then
        # Allow nginx (port 80) when using nginx
        if [ "$USE_NGINX" = "true" ]; then
            if ! $SUDO ufw status 2>/dev/null | grep -q "^80/tcp"; then
                $SUDO ufw allow 80/tcp >/dev/null 2>&1 || true
            fi
        fi
        # Always allow API port
        if ! $SUDO ufw status 2>/dev/null | grep -q "^${PORT}/tcp"; then
            $SUDO ufw allow ${PORT}/tcp >/dev/null 2>&1 || true
        fi
        success "Firewall rules added (ufw: ${PORT})"
    elif available firewall-cmd; then
        if [ "$USE_NGINX" = "true" ]; then
            $SUDO firewall-cmd --permanent --add-service=http >/dev/null 2>&1 || true
        fi
        $SUDO firewall-cmd --permanent --add-port=${PORT}/tcp >/dev/null 2>&1 || true
        $SUDO firewall-cmd --reload >/dev/null 2>&1 || true
        success "Firewall rules added (firewalld: ${PORT})"
    else
        warning "No firewall tool found (ufw/firewalld)."
        warning "Make sure port ${PORT} is open for LAN access."
    fi

    success "Installation complete!"
}

install_darwin() {
    status "Installing HeraMind on macOS..."

    # Create directories
    $SUDO mkdir -p "$INSTALL_DIR"
    mkdir -p "$DATA_DIR"

    # Download and extract
    BINARY_FILE="heramind-server-darwin-${ARCH}.tar.gz"
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/v${VERSION}/${BINARY_FILE}"

    status "Downloading HeraMind v${VERSION} for ${OS}/${ARCH}..."
    TEMP_DIR=$(mktemp -d)

    if ! curl -fSL --progress-bar "$DOWNLOAD_URL" -o "$TEMP_DIR/heramind.tar.gz"; then
        error "Failed to download from $DOWNLOAD_URL"
    fi

    status "Extracting..."
    tar xzf "$TEMP_DIR/heramind.tar.gz" -C "$TEMP_DIR"

    # Stop existing service before upgrading
    launchctl unload ~/Library/LaunchAgents/com.heramind.server.plist 2>/dev/null || true

    # Install binary
    status "Installing binary to $INSTALL_DIR..."
    $SUDO install -m 755 "$TEMP_DIR/heramind" "$INSTALL_DIR/heramind"

    # Install extension runner if present
    if [ -f "$TEMP_DIR/heramind-extension-runner" ]; then
        $SUDO install -m 755 "$TEMP_DIR/heramind-extension-runner" "$INSTALL_DIR/heramind-extension-runner"
        success "Extension runner installed"
    fi

    # Download frontend for macOS
    if [ "$NO_WEB" = "true" ]; then
        status "Skipping frontend (NO_WEB=true). Backend-only deployment."
    else
        WEB_FILE="heramind-web-${VERSION}.tar.gz"
        WEB_URL="https://github.com/${REPO}/releases/download/v${VERSION}/${WEB_FILE}"

        status "Downloading frontend..."
        if curl -fSL --progress-bar "$WEB_URL" -o "$TEMP_DIR/heramind-web.tar.gz" 2>/dev/null; then
            $SUDO mkdir -p "$WEB_DIR"
            $SUDO tar xzf "$TEMP_DIR/heramind-web.tar.gz" -C "$WEB_DIR"
            success "Frontend installed to $WEB_DIR"
        else
            warning "Frontend package not found. Web UI will show a placeholder page."
            warning "You can manually download it from the release page."
        fi
    fi

    # Create launchd plist for macOS
    if [ "$NO_SERVICE" != "true" ]; then
        status "Installing launchd service..."
        PLIST_PATH="$HOME/Library/LaunchAgents/com.heramind.server.plist"
        mkdir -p "$(dirname "$PLIST_PATH")"

        cat > "$PLIST_PATH" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.heramind.server</string>
    <key>ProgramArguments</key>
    <array>
        <string>${INSTALL_DIR}/heramind</string>
        <string>serve</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>EnvironmentVariables</key>
    <dict>
        <key>RUST_LOG</key>
        <string>info</string>
        <key>HERAMIND_WEB_DIR</key>
        <string>${WEB_DIR}</string>
        <key>HERAMIND_API_BASE</key>
        <string>http://localhost:${PORT}/api</string>
    </dict>
    <key>StandardOutPath</key>
    <string>${DATA_DIR}/heramind.log</string>
    <key>StandardErrorPath</key>
    <string>${DATA_DIR}/heramind.log</string>
</dict>
</plist>
EOF
        success "Launchd service installed"
    fi

    success "Installation complete!"
}

print_post_install() {
    echo ""
    echo "${BOLD}═══════════════════════════════════════════════════════════${NC}"
    echo "${BOLD}  HeraMind v${VERSION} installed successfully!${NC}"
    echo "${BOLD}═══════════════════════════════════════════════════════════${NC}"
    echo ""
    echo "Binary location: ${INSTALL_DIR}/heramind"
    echo "Data directory:  ${DATA_DIR}"
    if [ "$NO_WEB" != "true" ]; then
        echo "Frontend:        ${WEB_DIR}"
    else
        echo "Frontend:        skipped (backend-only)"
    fi
    echo ""

    if [ "$OS" = "linux" ]; then
        # Get LAN IP for display
        LAN_IP=$(hostname -I 2>/dev/null | awk '{print $1}')

        if [ "$NO_SERVICE" != "true" ]; then
            echo "Restarting HeraMind service..."
            $SUDO systemctl restart heramind || true
            sleep 1

            # Check if service is running
            if $SUDO systemctl is-active --quiet heramind 2>/dev/null; then
                success "HeraMind service is running"
            else
                warning "HeraMind service may not have started. Check: sudo journalctl -u heramind"
            fi

            echo ""
            echo "Service commands:"
            echo "  Status:  sudo systemctl status heramind"
            echo "  Stop:    sudo systemctl stop heramind"
            echo "  Restart: sudo systemctl restart heramind"
            echo "  Logs:    sudo journalctl -u heramind -f"
            echo ""
            echo "Access the application:"
            if [ "$USE_NGINX" = "true" ] && available nginx; then
                echo "  Web UI:  ${BOLD}http://${LAN_IP:-localhost}${NC} (nginx)"
                echo "  Direct:  http://${LAN_IP:-localhost}:${PORT}"
            else
                echo "  Web UI:  ${BOLD}http://${LAN_IP:-localhost}:${PORT}${NC}"
            fi
            echo "  API:     http://${LAN_IP:-localhost}:${PORT}/api"
            echo "  Docs:    http://${LAN_IP:-localhost}:${PORT}/api/docs"
        else
            echo "To start HeraMind:"
            echo "  ${INSTALL_DIR}/heramind serve"
            echo ""
            echo "Access:  http://${LAN_IP:-localhost}:${PORT}/api"
        fi
    elif [ "$OS" = "darwin" ]; then
        if [ "$NO_SERVICE" != "true" ]; then
            echo "Restarting HeraMind service..."
            launchctl unload ~/Library/LaunchAgents/com.heramind.server.plist 2>/dev/null || true
            launchctl load ~/Library/LaunchAgents/com.heramind.server.plist 2>/dev/null || true

            echo ""
            echo "Service commands:"
            echo "  Stop:   launchctl unload ~/Library/LaunchAgents/com.heramind.server.plist"
            echo "  Start:  launchctl load ~/Library/LaunchAgents/com.heramind.server.plist"
            echo "  Logs:   tail -f ${DATA_DIR}/heramind.log"
        else
            echo "To start HeraMind:"
            echo "  ${INSTALL_DIR}/heramind serve"
        fi
        echo ""
        echo "Access the application:"
        echo "  Web UI:  http://localhost:${PORT}"
        echo "  API:     http://localhost:${PORT}/api"
        echo "  Docs:    http://localhost:${PORT}/api/docs"
    fi

    echo ""
    echo "Documentation: https://github.com/camthink-ai/HeraMind"
    echo ""
}

main() {
    echo ""
    echo "${BOLD}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo "${BOLD}║           HeraMind Edge AI Platform Installer             ║${NC}"
    echo "${BOLD}╚═══════════════════════════════════════════════════════════╝${NC}"
    echo ""

    # Check dependencies
    require curl

    # Detect system
    get_os
    get_arch
    status "Detected: ${OS}/${ARCH}"

    # Get version
    if [ -z "$VERSION" ]; then
        get_latest_version
    fi
    status "Installing version: ${VERSION}"

    # Detect sudo
    detect_sudo

    # Install
    case "$OS" in
        linux) install_linux ;;
        darwin) install_darwin ;;
    esac

    print_post_install
}

main "$@"
