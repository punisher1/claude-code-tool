#!/usr/bin/env bash
# claude-code-tool (cct) installer
# Usage: curl -fsSL https://raw.githubusercontent.com/punisher1/claude-code-tool/main/install.sh | bash
#        curl -fsSL ... | bash -s -- --version 0.1.5
#        ./install.sh --version 0.1.5 --proxy http://127.0.0.1:11224

set -euo pipefail

REPO="punisher1/claude-code-tool"
BINARY_NAME="cct"
GITHUB_API_BASE="https://api.github.com/repos/${REPO}/releases"

# CLI 参数默认值
REQUESTED_VERSION=""
INSTALL_DIR=""
CURL_PROXY="${HTTPS_PROXY:-${https_proxy:-}}"
SKIP_CHECKSUM=0
CHECKSUM_OK=0  # checksum 是否成功获取且可用，0=否（降级跳过校验）

# ── 输出函数 ──

info()  { printf '\033[0;32m[INFO]\033[0m  %s\n' "$*"; }
warn()  { printf '\033[1;33m[WARN]\033[0m  %s\n' "$*"; }
error() { printf '\033[0;31m[ERROR]\033[0m %s\n' "$*" >&2; exit 1; }

print_banner() {
    echo "============================================"
    echo "  claude-code-tool (cct) Installer"
    echo "============================================"
    echo ""
}

# ── 参数解析 ──

show_help() {
    cat <<'EOF'
Usage: install.sh [options]

Options:
  --version VERSION    Install a specific version (e.g., 0.1.5)
  --install-dir DIR    Override installation directory
  --proxy URL          Use HTTP proxy for downloads
  --skip-checksum      Skip SHA256 checksum verification
  -h, --help           Show this help message

Environment variables:
  HTTPS_PROXY          HTTP proxy URL
EOF
    exit 0
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --version)      REQUESTED_VERSION="$2"; shift 2 ;;
            --install-dir)  INSTALL_DIR="$2"; shift 2 ;;
            --proxy)        CURL_PROXY="$2"; shift 2 ;;
            --skip-checksum) SKIP_CHECKSUM=1; shift ;;
            -h|--help)      show_help ;;
            *)              error "Unknown option: $1. Use --help for usage." ;;
        esac
    done
}

# ── 下载工具 ──

curl_cmd() {
    local proxy_args=()
    if [[ -n "$CURL_PROXY" ]]; then
        proxy_args=(--proxy "$CURL_PROXY")
    fi
    command curl --connect-timeout 30 --max-time 300 "${proxy_args[@]}" "$@"
}

if command -v curl >/dev/null 2>&1; then
    download() { curl_cmd -fSL "$1" -o "$2"; }
elif command -v wget >/dev/null 2>&1; then
    download() {
        local proxy_args=()
        if [[ -n "$CURL_PROXY" ]]; then
            proxy_args=(--proxy-on)
        fi
        command wget --quiet --timeout=30 "${proxy_args[@]}" -O "$2" "$1"
    }
else
    error "Requires curl or wget for downloading."
fi

# ── 平台检测 ──

detect_platform() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux)  PLATFORM="Linux" ;;
        Darwin) PLATFORM="Darwin" ;;
        *)      error "Unsupported operating system: $os" ;;
    esac

    case "$arch" in
        x86_64|amd64)  ARCH_SUFFIX="x86_64" ;;
        aarch64|arm64) ARCH_SUFFIX="aarch64" ;;
        *)             error "Unsupported architecture: $arch" ;;
    esac

    # Linux 目前只有 x86_64 构建产物
    if [[ "$PLATFORM" == "Linux" && "$ARCH_SUFFIX" == "aarch64" ]]; then
        error "Linux aarch64 is not currently supported."
    fi

    ARTIFACT_NAME="cct-${PLATFORM}-${ARCH_SUFFIX}.tar.gz"
    info "Platform: ${PLATFORM} ${ARCH_SUFFIX}"
}

# ── 安装目录 ──

determine_install_dir() {
    if [[ -z "$INSTALL_DIR" ]]; then
        INSTALL_DIR="${HOME}/.local/bin"
    fi
    mkdir -p "$INSTALL_DIR"
}

# ── 版本解析 ──

resolve_version() {
    local api_url
    if [[ -n "$REQUESTED_VERSION" ]]; then
        VERSION="v${REQUESTED_VERSION#v}"
        api_url="${GITHUB_API_BASE}/tags/${VERSION}"
    else
        api_url="${GITHUB_API_BASE}/latest"
    fi

    info "Querying GitHub API for release information..."

    RELEASE_JSON=$(curl_cmd -fsSL "$api_url") || error "Failed to fetch release info. Check network or use --version."

    VERSION=$(echo "$RELEASE_JSON" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    [[ -z "$VERSION" ]] && error "Could not determine version from API response."

    info "Installing version: $VERSION"
}

# ── 下载和校验 ──

download_artifacts() {
    local download_base archive_url checksum_url

    download_base="https://github.com/${REPO}/releases/download/${VERSION}"
    archive_url="${download_base}/${ARTIFACT_NAME}"
    checksum_url="${download_base}/${ARTIFACT_NAME}.sha256"

    TEMP_DIR=$(mktemp -d)
    trap 'rm -rf "$TEMP_DIR"' EXIT

    ARCHIVE_PATH="${TEMP_DIR}/${ARTIFACT_NAME}"
    CHECKSUM_PATH="${TEMP_DIR}/${ARTIFACT_NAME}.sha256"
    CHECKSUM_OK=0

    # archive 是必需的：下载失败/为空 → 明确报错（带 URL）
    info "Downloading ${ARTIFACT_NAME}..."
    download "$archive_url" "$ARCHIVE_PATH" \
        || error "Failed to download ${archive_url}"
    [[ -s "$ARCHIVE_PATH" ]] \
        || error "Downloaded archive is empty or missing: ${ARCHIVE_PATH}"

    # checksum 是可选的：下载/文件失败 → 置 CHECKSUM_OK=0 并 warn，不中断安装
    # 用 [[ -s ]] 立即检查文件存在且非空，可捕获「curl 报成功但文件读不到」的场景
    if [[ "$SKIP_CHECKSUM" != "1" ]]; then
        info "Downloading checksum..."
        if download "$checksum_url" "$CHECKSUM_PATH" 2>/dev/null \
           && [[ -s "$CHECKSUM_PATH" ]]; then
            CHECKSUM_OK=1
        else
            warn "Could not download checksum; skipping verification."
        fi
    fi
}

verify_checksum() {
    # 未启用校验、或 checksum 未成功获取 → 降级跳过（与 cct update 行为一致）
    if [[ "$SKIP_CHECKSUM" == "1" || "$CHECKSUM_OK" != "1" ]]; then
        warn "Skipping checksum verification."
        return
    fi

    [[ -f "$CHECKSUM_PATH" ]] || { warn "Checksum file missing; skipping."; return; }

    local expected_hash actual_hash
    expected_hash=$(awk '{print $1}' "$CHECKSUM_PATH")
    if [[ -z "$expected_hash" ]]; then
        warn "Could not read expected checksum; skipping."
        return
    fi

    if command -v sha256sum >/dev/null 2>&1; then
        actual_hash=$(sha256sum "$ARCHIVE_PATH" | awk '{print $1}')
    elif command -v shasum >/dev/null 2>&1; then
        actual_hash=$(shasum -a 256 "$ARCHIVE_PATH" | awk '{print $1}')
    else
        warn "Neither sha256sum nor shasum found. Skipping verification."
        return
    fi

    if [[ "$actual_hash" != "$expected_hash" ]]; then
        # 降级：不中断安装，仅警告（与 cct update 一致）
        warn "Checksum mismatch (expected ${expected_hash}, got ${actual_hash})."
        warn "The archive may be corrupted or tampered; proceeding anyway."
        return
    fi

    info "Checksum verified."
}

# ── 安装 ──

install_binary() {
    info "Extracting archive..."
    tar -xzf "$ARCHIVE_PATH" -C "$TEMP_DIR"

    local binary_path="${TEMP_DIR}/${BINARY_NAME}"
    [[ ! -f "$binary_path" ]] && error "Binary not found in archive."

    chmod +x "$binary_path"

    if [[ -f "${INSTALL_DIR}/${BINARY_NAME}" ]]; then
        info "Replacing existing installation at ${INSTALL_DIR}/${BINARY_NAME}"
    fi

    mv "$binary_path" "${INSTALL_DIR}/${BINARY_NAME}"

    # macOS: 移除 quarantine 属性
    if [[ "$PLATFORM" == "Darwin" ]]; then
        xattr -d com.apple.quarantine "${INSTALL_DIR}/${BINARY_NAME}" 2>/dev/null || true
    fi

    info "Installed ${BINARY_NAME} to ${INSTALL_DIR}/${BINARY_NAME}"
}

# ── 安装后验证 ──

shell_profile_path() {
    local shell_name
    shell_name="$(basename "${SHELL:-}")"

    case "$shell_name" in
        zsh)  printf '%s\n' "${HOME}/.zshrc" ;;
        bash) printf '%s\n' "${HOME}/.bashrc" ;;
        *)    printf '%s\n' "${HOME}/.profile" ;;
    esac
}

ensure_path() {
    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*)
            info "${INSTALL_DIR} is already in PATH."
            return
            ;;
    esac

    export PATH="${INSTALL_DIR}:${PATH}"

    local profile
    profile="$(shell_profile_path)"

    if [[ -f "$profile" ]] && grep -Fq "$INSTALL_DIR" "$profile"; then
        warn "${INSTALL_DIR} is already referenced in ${profile}. Open a new terminal if cct is not found."
        return
    fi

    if {
        printf '\n# Added by cct installer\n'
        printf 'export PATH="%s:$PATH"\n' "$INSTALL_DIR"
    } >> "$profile"; then
        info "Added ${INSTALL_DIR} to ${profile}. Open a new terminal for it to take effect."
    else
        warn "Could not update PATH automatically."
        warn "Add the following to your shell profile:"
        warn "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    fi
}

verify_installation() {
    if command -v "$BINARY_NAME" >/dev/null 2>&1; then
        local installed_version
        installed_version=$("$BINARY_NAME" --version 2>/dev/null || echo "unknown")
        info "Verification: ${BINARY_NAME} ${installed_version}"
    else
        warn "${BINARY_NAME} is installed but not in your PATH."
        warn "Add the following to your shell profile:"
        warn "  export PATH=\"${INSTALL_DIR}:\$PATH\""
        echo ""
        warn "Or run now with: ${INSTALL_DIR}/${BINARY_NAME}"
    fi
}

# ── 主流程 ──

main() {
    parse_args "$@"
    print_banner
    detect_platform
    determine_install_dir
    resolve_version
    download_artifacts
    verify_checksum
    install_binary
    ensure_path
    verify_installation

    echo ""
    info "Installation complete! Run '${BINARY_NAME} --help' to get started."
}

main "$@"
