#!/bin/bash
# Generate and export Apple signing certificate for GitHub Actions
# Run this on macOS with your Apple Developer account

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo -e "${BLUE}==>${NC} $1"
}

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    log_error "This script must be run on macOS"
    exit 1
fi

# Check for existing certificates
log_step "Checking for existing Developer ID certificates..."
EXISTING_CERTS=$(security find-identity -v -p codesigning 2>/dev/null | grep "Developer ID Application" || true)

if [ -z "$EXISTING_CERTS" ]; then
    log_warn "No Developer ID Application certificates found"
    echo ""
    echo "You have two options:"
    echo ""
    echo "1. ${GREEN}Apple ID Signing (Free)${NC}"
    echo "   - Uses your Apple ID for ad-hoc signing"
    echo "   - Users need to right-click to open"
    echo "   - No certificate needed"
    echo ""
    echo "2. ${YELLOW}Developer ID Certificate ($99/year)${NC}"
    echo "   - Requires Apple Developer Program membership"
    echo "   - Better UX - no warnings for users"
    echo "   - Create at: https://developer.apple.com/account/resources/certificates/list"
    echo ""
    
    read -p "Do you want to use Apple ID signing (free)? (y/n): " USE_APPLE_ID
    
    if [[ "$USE_APPLE_ID" == "y" || "$USE_APPLE_ID" == "Y" ]]; then
        log_info "Using Apple ID signing mode"
        log_info "No certificate export needed - will use ad-hoc signing"
        echo ""
        log_info "For GitHub Actions, you still need:"
        log_info "  - APPLE_ID: Your Apple ID email"
        log_info "  - APPLE_PASSWORD: App-specific password"
        log_info "  - APPLE_TEAM_ID: Your Team ID (optional)"
        echo ""
        log_info "Get app-specific password at: https://appleid.apple.com"
        exit 0
    else
        log_error "Developer ID certificate required for this option"
        log_info "Please:"
        log_info "  1. Join Apple Developer Program: https://developer.apple.com/programs/"
        log_info "  2. Create a Developer ID Application certificate"
        log_info "  3. Run this script again"
        exit 1
    fi
fi

# Show existing certificates
log_step "Found certificates:"
echo "$EXISTING_CERTS"
echo ""

# Let user choose certificate
CERT_COUNT=$(echo "$EXISTING_CERTS" | wc -l | tr -d ' ')
if [ "$CERT_COUNT" -gt 1 ]; then
    log_info "Multiple certificates found. Using the first one."
fi

# Extract certificate identity
CERT_IDENTITY=$(echo "$EXISTING_CERTS" | head -1 | sed 's/.*"\(.*\)".*//')
log_info "Using certificate: $CERT_IDENTITY"

# Export certificate
log_step "Exporting certificate..."
CERT_FILE="$PROJECT_DIR/.github/scripts/certificate.p12"
CERT_BASE64_FILE="$PROJECT_DIR/.github/scripts/certificate-base64.txt"

# Get temporary password
CERT_PASSWORD=$(openssl rand -base64 32)
log_info "Generated certificate password"

# Export to P12
security find-certificate -c "$CERT_IDENTITY" -p |     openssl pkcs12 -export -out "$CERT_FILE"     -passout "pass:$CERT_PASSWORD" 2>/dev/null || {
    log_error "Failed to export certificate"
    log_info "You may need to allow Keychain access in System Preferences"
    exit 1
}

# Convert to base64
base64 -i "$CERT_FILE" -o "$CERT_BASE64_FILE"

# Get Team ID
TEAM_ID=$(security find-certificate -c "$CERT_IDENTITY" -p |     openssl x509 -noout -text |     grep -A1 "Subject Organizational Unit" |     tail -1 | sed 's/.*\([A-Z0-9]{10}\).*//' || echo "")

log_step "Certificate exported successfully!"
echo ""
log_info "Files created:"
echo "  - $CERT_FILE"
echo "  - $CERT_BASE64_FILE"
echo ""
log_info "Add these to your GitHub repository Secrets:"
echo ""
echo -e "${GREEN}APPLE_CERTIFICATE${NC}"
cat "$CERT_BASE64_FILE" | pbcopy
echo "  (Content copied to clipboard)"
echo ""
echo -e "${GREEN}APPLE_CERTIFICATE_PASSWORD${NC}"
echo "  $CERT_PASSWORD"
echo ""
if [ -n "$TEAM_ID" ]; then
    echo -e "${GREEN}APPLE_TEAM_ID${NC}"
    echo "  $TEAM_ID"
    echo ""
fi
echo -e "${GREEN}APPLE_ID${NC}"
echo "  Your Apple ID email"
echo ""
echo -e "${GREEN}APPLE_PASSWORD${NC}"
echo "  App-specific password from https://appleid.apple.com"
echo ""

# Cleanup
rm -f "$CERT_FILE"
log_info "Temporary P12 file removed"

log_step "Next steps:"
log_info "1. Go to: https://github.com/camthink-ai/HeraMind/settings/secrets/actions"
log_info "2. Add each secret from above"
log_info "3. Push a new tag to trigger build: git tag v0.3.0 && git push origin v0.3.0"
