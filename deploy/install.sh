#!/usr/bin/env bash
# =============================================================================
# DDNS Remake — Installer (Linux & macOS)
# =============================================================================
set -euo pipefail

# ── Constants ─────────────────────────────────────────────────────────────────
REPO="Derrick-Program/DDNS-Remake"
INSTALL_DIR="/opt/duacodie"
BIN_DIR="${INSTALL_DIR}/bin"
CONFIG_DIR="${INSTALL_DIR}/config"
DATA_DIR="${INSTALL_DIR}/data"
LOG_DIR="/var/log/duacodie"
SERVICE_USER="duacodie"
SERVICE_GROUP="duacodie"

# ── Colors ────────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

info()    { echo -e "${BLUE}[INFO]${NC} $*"; }
success() { echo -e "${GREEN}[OK]${NC}   $*"; }
warn()    { echo -e "${YELLOW}[WARN]${NC} $*"; }
error()   { echo -e "${RED}[ERR]${NC}  $*" >&2; }
die()     { error "$*"; exit 1; }
bold()    { echo -e "${BOLD}$*${NC}"; }

# ── Version resolution ────────────────────────────────────────────────────────
resolve_version() {
    if [[ -n "${DDNS_VERSION:-}" ]]; then
        VERSION="${DDNS_VERSION}"
        info "Using specified version: ${VERSION}"
    else
        info "Fetching latest release version from GitHub ..."
        VERSION=$(curl -fsSL \
            -H "Accept: application/vnd.github+json" \
            "https://api.github.com/repos/${REPO}/releases/latest" \
            | grep -m1 '"tag_name"' \
            | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')
        if [[ -z "${VERSION}" ]]; then
            die "Could not fetch latest version. Set DDNS_VERSION env var to specify one manually (e.g. DDNS_VERSION=v0.1.1 sudo ./install.sh)"
        fi
        success "Latest version: ${VERSION}"
    fi
    BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"
}

# ── Privilege check ───────────────────────────────────────────────────────────
require_root() {
    if [[ "$EUID" -ne 0 ]]; then
        die "This installer must be run as root. Try: sudo $0"
    fi
}

# ── Dependency check ──────────────────────────────────────────────────────────
check_deps() {
    local missing=()
    for cmd in curl; do
        command -v "$cmd" &>/dev/null || missing+=("$cmd")
    done
    if [[ ${#missing[@]} -gt 0 ]]; then
        die "Missing required tools: ${missing[*]}"
    fi
}

# ── OS / Arch detection ───────────────────────────────────────────────────────
detect_platform() {
    OS=$(uname -s)
    ARCH=$(uname -m)

    case "${OS}" in
        Linux)
            PLATFORM="linux"
            case "${ARCH}" in
                x86_64)  ARCH_LABEL="x64-musl" ;;
                #aarch64) ARCH_LABEL="arm64-musl" ;;
                *) die "Unsupported Linux architecture: ${ARCH}" ;;
            esac
            ;;
        Darwin)
            PLATFORM="macos"
            case "${ARCH}" in
                arm64)  ARCH_LABEL="arm64" ;;
                x86_64) ARCH_LABEL="intel" ;;
                *) die "Unsupported macOS architecture: ${ARCH}" ;;
            esac
            ;;
        *) die "Unsupported OS: ${OS}. Please use the Windows installer (install.ps1) on Windows." ;;
    esac

    CLIENT_BINARY="ddns-client-${PLATFORM}-${ARCH_LABEL}"
    SERVER_BINARY="ddns-server-${PLATFORM}-${ARCH_LABEL}"
}

# ── Download helper ───────────────────────────────────────────────────────────
download_binary() {
    local name="$1"
    local dest="$2"
    local url="${BASE_URL}/${name}"

    info "Downloading ${name} ..."
    if ! curl -fsSL --progress-bar -o "${dest}" "${url}"; then
        die "Failed to download ${url}"
    fi
    chmod +x "${dest}"
    success "Downloaded → ${dest}"
}

# ── Create system user ────────────────────────────────────────────────────────
create_service_user() {
    if id "${SERVICE_USER}" &>/dev/null; then
        info "Service user '${SERVICE_USER}' already exists."
        return
    fi

    info "Creating system user '${SERVICE_USER}' ..."
    if [[ "${PLATFORM}" == "linux" ]]; then
        useradd --system --no-create-home --shell /usr/sbin/nologin \
            --home-dir "${INSTALL_DIR}" "${SERVICE_USER}"
    else
        # macOS
        local uid
        uid=$(dscl . -list /Users UniqueID | awk '{print $2}' | sort -n | tail -1)
        uid=$((uid + 1))
        dscl . -create "/Users/${SERVICE_USER}"
        dscl . -create "/Users/${SERVICE_USER}" UserShell /usr/bin/false
        dscl . -create "/Users/${SERVICE_USER}" UniqueID "${uid}"
        dscl . -create "/Users/${SERVICE_USER}" PrimaryGroupID 20
        dscl . -create "/Users/${SERVICE_USER}" NFSHomeDirectory /var/empty
    fi
    success "Created user '${SERVICE_USER}'."
}

# ── Directory setup ───────────────────────────────────────────────────────────
setup_directories() {
    local dirs=("${BIN_DIR}" "${CONFIG_DIR}" "${DATA_DIR}" "${LOG_DIR}")
    for d in "${dirs[@]}"; do
        mkdir -p "${d}"
    done
    chown -R "${SERVICE_USER}:${SERVICE_GROUP}" "${INSTALL_DIR}" "${LOG_DIR}" 2>/dev/null || \
        chown -R "${SERVICE_USER}" "${INSTALL_DIR}" "${LOG_DIR}"
    success "Directories created under ${INSTALL_DIR}"
}

# ── Interactive prompt helper ─────────────────────────────────────────────────
prompt() {
    local var="$1"
    local msg="$2"
    local default="${3:-}"
    local value

    if [[ -n "${default}" ]]; then
        read -rp "  ${msg} [${default}]: " value </dev/tty
        value="${value:-${default}}"
    else
        read -rp "  ${msg}: " value </dev/tty
        while [[ -z "${value}" ]]; do
            echo "  (Value cannot be empty)"
            read -rp "  ${msg}: " value </dev/tty
        done
    fi
    printf -v "${var}" '%s' "${value}"
}

# ── Generate random string ────────────────────────────────────────────────────
random_hex() {
    local len="${1:-32}"
    LC_ALL=C tr -dc 'a-f0-9' </dev/urandom 2>/dev/null | head -c "${len}" || true
}

# =============================================================================
# SERVER INSTALLER
# =============================================================================
install_server() {
    bold "\n=== Installing DDNS Server ==="

    require_root
    detect_platform
    check_deps
    create_service_user
    setup_directories
    download_binary "${SERVER_BINARY}" "${BIN_DIR}/ddns-server"
    echo ""
    bold "Server Configuration"
    echo "  (Press Enter to accept defaults)"
    echo ""
    prompt SERVER_HOST      "Listen host"              "127.0.0.1"
    prompt SERVER_PORT      "Listen port"              "8698"
    prompt CF_API_KEY       "Cloudflare API key"       ""
    prompt DB_PATH          "Database path (DATABASE_URL)" "sqlite://${DATA_DIR}/ddns.db"
    local cfg_dir="${CONFIG_DIR}/duacodie/ddns"
    local cfg_file="${cfg_dir}/config.toml"
    mkdir -p "${cfg_dir}"
    cat > "${cfg_file}" <<EOF
# DDNS Server Configuration
# Generated by installer on $(date -u '+%Y-%m-%dT%H:%M:%SZ')
# Edit with: ddns-server config set <key> <value>

[server]
host = "${SERVER_HOST}"
port = ${SERVER_PORT}

[cloudflare]
api_key = "${CF_API_KEY}"

# Add DNS zones below, or use: ddns-server config zone-add <zone>
# [[zones]]
# name = "example.com"
EOF
    chown -R "${SERVICE_USER}" "${cfg_dir}"
    chmod 600 "${cfg_file}"
    success "Config written to ${cfg_file}"
    if [[ "${PLATFORM}" == "linux" ]]; then
        _setup_systemd_server "${DB_PATH}"
    else
        _setup_launchd_server "${DB_PATH}"
    fi
    echo ""
    success "DDNS Server installed successfully!"
    bold "\nNext steps:"
    echo "  1. Add a DNS zone:  ddns-server config zone-add <your-domain>"
    echo "  2. Add a user:      ddns-server server add-user -u <username>"
    echo "  3. Review config:   ${cfg_file}"
    echo "  4. Check status:    $([ "${PLATFORM}" = "linux" ] && echo "systemctl status duacodie-server" || echo "launchctl list com.duacodie.server")"
    warn "Note: JWT secret is auto-generated on each start. Tokens will be invalidated on restart."
}

# ── systemd (Linux) ───────────────────────────────────────────────────────────
_setup_systemd_server() {
    local db_url="$1"
    local unit_file="/etc/systemd/system/duacodie-server.service"
    info "Installing systemd service → ${unit_file}"

    cat > "${unit_file}" <<EOF
[Unit]
Description=Duacodie DDNS Server
Documentation=https://github.com/${REPO}
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=${SERVICE_USER}
Group=${SERVICE_GROUP}
WorkingDirectory=${INSTALL_DIR}
Environment=XDG_CONFIG_HOME=${CONFIG_DIR}
Environment=DATABASE_URL=${db_url}
ExecStart=${BIN_DIR}/ddns-server start
Restart=on-failure
RestartSec=10
StandardOutput=journal
StandardError=journal

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=${DATA_DIR} ${LOG_DIR} ${CONFIG_DIR}
PrivateTmp=true

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    systemctl enable duacodie-server
    systemctl start  duacodie-server
    success "Service enabled and started (systemd)"
}

# ── launchd (macOS) ───────────────────────────────────────────────────────────
_setup_launchd_server() {
    local db_url="$1"
    local plist_dir="/Library/LaunchDaemons"
    local plist_file="${plist_dir}/com.duacodie.server.plist"
    info "Installing launchd daemon → ${plist_file}"

    cat > "${plist_file}" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
    "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.duacodie.server</string>

    <key>ProgramArguments</key>
    <array>
        <string>${BIN_DIR}/ddns-server</string>
        <string>start</string>
    </array>

    <key>EnvironmentVariables</key>
    <dict>
        <key>XDG_CONFIG_HOME</key>
        <string>${CONFIG_DIR}</string>
        <key>DATABASE_URL</key>
        <string>${db_url}</string>
    </dict>

    <key>UserName</key>
    <string>${SERVICE_USER}</string>

    <key>WorkingDirectory</key>
    <string>${INSTALL_DIR}</string>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
    </dict>

    <key>StandardOutPath</key>
    <string>${LOG_DIR}/ddns-server.log</string>

    <key>StandardErrorPath</key>
    <string>${LOG_DIR}/ddns-server.err</string>
</dict>
</plist>
EOF

    launchctl load -w "${plist_file}"
    success "Service loaded (launchd)"
}

# =============================================================================
# CLIENT INSTALLER
# =============================================================================
install_client() {
    bold "\n=== Installing DDNS Client ==="

    require_root
    detect_platform
    check_deps
    create_service_user
    setup_directories

    download_binary "${CLIENT_BINARY}" "${BIN_DIR}/ddns-client"
    echo ""
    bold "Client Configuration"
    echo "  (Press Enter to accept defaults)"
    echo ""
    prompt SERVER_URL   "DDNS server URL"      "http://127.0.0.1:8698"
    prompt DEVICE_TOKEN "Device API token (from 'ddns-server auth exchange')" ""
    prompt CHECK_INTERVAL "IP check interval (seconds)" "300"
    local cfg_dir="${CONFIG_DIR}/duacodie/ddns-client"
    local cfg_file="${cfg_dir}/config.toml"
    mkdir -p "${cfg_dir}"
    cat > "${cfg_file}" <<EOF
# DDNS Client Configuration
# Generated by installer on $(date -u '+%Y-%m-%dT%H:%M:%SZ')

server_url = "${SERVER_URL}"
device_token = "${DEVICE_TOKEN}"
check_interval_secs = ${CHECK_INTERVAL}
domains = []
EOF
    chown -R "${SERVICE_USER}" "${cfg_dir}"
    chmod 600 "${cfg_file}"
    success "Config written to ${cfg_file}"

    # Setup service
    if [[ "${PLATFORM}" == "linux" ]]; then
        _setup_systemd_client
    else
        _setup_launchd_client
    fi

    echo ""
    success "DDNS Client installed successfully!"
    bold "\nNext steps:"
    echo "  1. Review config:  ${CONFIG_DIR}/duacodie/ddns-client/config.toml"
    echo "  2. Check status:   $([ "${PLATFORM}" = "linux" ] && echo "systemctl status duacodie-client" || echo "launchctl list com.duacodie.client")"
    echo "  3. View logs:      $([ "${PLATFORM}" = "linux" ] && echo "journalctl -u duacodie-client -f" || echo "tail -f ${LOG_DIR}/ddns-client.log")"
}

# ── systemd (Linux) ───────────────────────────────────────────────────────────
_setup_systemd_client() {
    local unit_file="/etc/systemd/system/duacodie-client.service"
    info "Installing systemd service → ${unit_file}"

    cat > "${unit_file}" <<EOF
[Unit]
Description=Duacodie DDNS Client
Documentation=https://github.com/${REPO}
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=${SERVICE_USER}
Group=${SERVICE_GROUP}
WorkingDirectory=${INSTALL_DIR}
Environment=XDG_CONFIG_HOME=${CONFIG_DIR}
ExecStart=${BIN_DIR}/ddns-client run
Restart=on-failure
RestartSec=10
StandardOutput=journal
StandardError=journal

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=${CONFIG_DIR} ${LOG_DIR}
PrivateTmp=true

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    systemctl enable duacodie-client
    systemctl start  duacodie-client
    success "Service enabled and started (systemd)"
}

# ── launchd (macOS) ───────────────────────────────────────────────────────────
_setup_launchd_client() {
    local plist_dir="/Library/LaunchDaemons"
    local plist_file="${plist_dir}/com.duacodie.client.plist"
    info "Installing launchd daemon → ${plist_file}"

    cat > "${plist_file}" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
    "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.duacodie.client</string>

    <key>ProgramArguments</key>
    <array>
        <string>${BIN_DIR}/ddns-client</string>
        <string>run</string>
    </array>

    <key>EnvironmentVariables</key>
    <dict>
        <key>XDG_CONFIG_HOME</key>
        <string>${CONFIG_DIR}</string>
    </dict>

    <key>UserName</key>
    <string>${SERVICE_USER}</string>

    <key>WorkingDirectory</key>
    <string>${INSTALL_DIR}</string>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
    </dict>

    <key>StandardOutPath</key>
    <string>${LOG_DIR}/ddns-client.log</string>

    <key>StandardErrorPath</key>
    <string>${LOG_DIR}/ddns-client.err</string>
</dict>
</plist>
EOF

    launchctl load -w "${plist_file}"
    success "Service loaded (launchd)"
}

# =============================================================================
# UNINSTALLER
# =============================================================================
uninstall() {
    bold "\n=== Uninstalling DDNS Remake ==="
    require_root

    detect_platform

    read -rp "  This will remove all DDNS binaries and services. Continue? [y/N]: " CONFIRM </dev/tty
    [[ "${CONFIRM}" == "y" || "${CONFIRM}" == "Y" ]] || { info "Aborted."; exit 0; }

    if [[ "${PLATFORM}" == "linux" ]]; then
        for svc in duacodie-server duacodie-client; do
            if systemctl is-active --quiet "${svc}" 2>/dev/null; then
                systemctl stop "${svc}"
            fi
            if systemctl is-enabled --quiet "${svc}" 2>/dev/null; then
                systemctl disable "${svc}"
            fi
            rm -f "/etc/systemd/system/${svc}.service"
        done
        systemctl daemon-reload
    else
        for label in com.duacodie.server com.duacodie.client; do
            local plist="/Library/LaunchDaemons/${label}.plist"
            if [[ -f "${plist}" ]]; then
                launchctl unload -w "${plist}" 2>/dev/null || true
                rm -f "${plist}"
            fi
        done
    fi

    rm -rf "${INSTALL_DIR}"
    rm -rf "${LOG_DIR}"

    # Remove service user and group
    if [[ "${PLATFORM}" == "linux" ]]; then
        if id "${SERVICE_USER}" &>/dev/null; then
            userdel "${SERVICE_USER}"
            success "Removed user '${SERVICE_USER}'."
        fi
        if getent group "${SERVICE_GROUP}" &>/dev/null; then
            groupdel "${SERVICE_GROUP}" 2>/dev/null || true
            success "Removed group '${SERVICE_GROUP}'."
        fi
    else
        if dscl . -read "/Users/${SERVICE_USER}" &>/dev/null 2>&1; then
            dscl . -delete "/Users/${SERVICE_USER}"
            success "Removed user '${SERVICE_USER}'."
        fi
    fi

    success "Uninstall complete."
}

# =============================================================================
# MAIN MENU
# =============================================================================
main() {
    resolve_version

    clear
    echo ""
    bold "╔════════════════════════════════════╗"
    bold "║   DDNS Remake Installer ${VERSION}    ║"
    bold "╚════════════════════════════════════╝"
    echo ""
    echo "  What would you like to do?"
    echo ""
    echo "  [1] Install DDNS Server"
    echo "  [2] Install DDNS Client"
    echo "  [3] Uninstall DDNS Remake"
    echo "  [4] Exit"
    echo ""
    read -rp "  Select [1-4]: " CHOICE </dev/tty
    echo ""

    case "${CHOICE}" in
        1) install_server ;;
        2) install_client ;;
        3) uninstall ;;
        4) info "Bye!"; exit 0 ;;
        *) die "Invalid choice: ${CHOICE}" ;;
    esac
}

main "$@"
