#!/usr/bin/env bash
# install.sh — samsara 安装脚本
# 用法：curl -fsSL https://raw.githubusercontent.com/mocikadev/mocika-samsara/main/install.sh | bash
# 或：   SAMSARA_VERSION=v0.1.0 bash install.sh

set -euo pipefail

REPO="mocikadev/mocika-samsara"
BINARY="samsara"
INSTALL_DIR="${SAMSARA_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${SAMSARA_VERSION:-latest}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
RESET='\033[0m'

info()  { printf "${BOLD}info${RESET}  %s\n" "$*"; }
ok()    { printf "${GREEN}ok${RESET}    %s\n" "$*"; }
warn()  { printf "${YELLOW}warn${RESET}  %s\n" "$*"; }
die()   { printf "${RED}error${RESET} %s\n" "$*" >&2; exit 1; }

# ---------------------------------------------------------------------------
# i18n：根据 $LANG 环境变量选择语言（zh_* → 中文，其余 → 英文）
# ---------------------------------------------------------------------------
case "${LANG:-}" in
  zh_*)
    MSG_TITLE="安装 samsara — AI Agent 知识管理 CLI"
    MSG_PLATFORM="平台    ："
    MSG_VERSION="版本    ："
    MSG_INSTALL_DIR="安装目录："
    MSG_DOWNLOADING="下载中..."
    MSG_VERIFYING="校验 SHA256..."
    MSG_CHECKSUM_OK="SHA256 校验通过"
    MSG_CHECKSUM_SKIP_DL="无法下载 SHA256SUMS.txt，跳过校验。"
    MSG_CHECKSUM_SKIP_MISSING="SHA256SUMS.txt 中未找到对应条目，跳过校验。"
    MSG_CHECKSUM_SKIP_CMD="未找到 sha256sum / shasum，跳过校验。"
    MSG_CHECKSUM_FAIL="SHA256 校验失败！\n  预期: %s\n  实际: %s"
    MSG_NO_DOWNLOADER="需要 curl 或 wget 之一，但均未找到。"
    MSG_DOWNLOAD_FAIL="下载失败，请检查网络或版本号是否正确。"
    MSG_INSTALLED="已安装："
    MSG_PATH_WARN="%s 不在 \$PATH 中，请将以下内容加入 ~/.bashrc 或 ~/.zshrc："
    MSG_DONE="完成！"
    MSG_HINT="运行 %ssamsara --help%s 开始使用。"
    MSG_UNSUPPORTED_ARCH="不支持的架构：%s（%s）"
    MSG_UNSUPPORTED_OS="不支持的操作系统：%s（仅支持 Linux / macOS；Windows 请用 install.ps1）"
    MSG_SKM_FOUND="skm 已安装："
    MSG_SKM_INSTALLING="未检测到 skm，正在自动安装..."
    MSG_SKM_INSTALL_OK="skm 安装完成。"
    MSG_SKM_INSTALL_FAIL="skm 自动安装失败，请手动安装：https://github.com/mocikadev/mocika-skills-cli"
    ;;
  *)
    MSG_TITLE="Installing samsara — AI Agent knowledge management CLI"
    MSG_PLATFORM="Platform   :"
    MSG_VERSION="Version    :"
    MSG_INSTALL_DIR="Install dir:"
    MSG_DOWNLOADING="Downloading..."
    MSG_VERIFYING="Verifying SHA256..."
    MSG_CHECKSUM_OK="SHA256 checksum verified"
    MSG_CHECKSUM_SKIP_DL="Could not download SHA256SUMS.txt, skipping verification."
    MSG_CHECKSUM_SKIP_MISSING="No matching entry in SHA256SUMS.txt, skipping verification."
    MSG_CHECKSUM_SKIP_CMD="sha256sum / shasum not found, skipping verification."
    MSG_CHECKSUM_FAIL="SHA256 mismatch!\n  expected: %s\n  actual:   %s"
    MSG_NO_DOWNLOADER="curl or wget is required but neither was found."
    MSG_DOWNLOAD_FAIL="Download failed. Check your network or the version string."
    MSG_INSTALLED="Installed:"
    MSG_PATH_WARN="%s is not in \$PATH. Add the following to ~/.bashrc or ~/.zshrc:"
    MSG_DONE="Done!"
    MSG_HINT="Run %ssamsara --help%s to get started."
    MSG_UNSUPPORTED_ARCH="Unsupported architecture: %s (%s)"
    MSG_UNSUPPORTED_OS="Unsupported OS: %s (only Linux / macOS are supported; Windows: use install.ps1)"
    MSG_SKM_FOUND="skm already installed:"
    MSG_SKM_INSTALLING="skm not found, installing automatically..."
    MSG_SKM_INSTALL_OK="skm installed successfully."
    MSG_SKM_INSTALL_FAIL="skm auto-install failed. Install manually: https://github.com/mocikadev/mocika-skills-cli"
    ;;
esac

# ---------------------------------------------------------------------------

detect_target() {
  local os arch
  os=$(uname -s)
  arch=$(uname -m)

  case "$os" in
    Linux)
      case "$arch" in
        x86_64)          echo "linux-amd64" ;;
        aarch64 | arm64) echo "linux-arm64" ;;
        *)               die "$(printf "$MSG_UNSUPPORTED_ARCH" "$arch" "Linux")" ;;
      esac
      ;;
    Darwin)
      case "$arch" in
        x86_64)          echo "macos-amd64" ;;
        arm64)           echo "macos-arm64" ;;
        *)               die "$(printf "$MSG_UNSUPPORTED_ARCH" "$arch" "macOS")" ;;
      esac
      ;;
    *)
      die "$(printf "$MSG_UNSUPPORTED_OS" "$os")"
      ;;
  esac
}

download() {
  local url="$1" dest="$2"
  if command -v curl > /dev/null 2>&1; then
    curl -fsSL --retry 3 --retry-delay 2 "$url" -o "$dest"
  elif command -v wget > /dev/null 2>&1; then
    wget -q --tries=3 "$url" -O "$dest"
  else
    die "$MSG_NO_DOWNLOADER"
  fi
}

resolve_url() {
  local name="$1"
  if [ "$VERSION" = "latest" ]; then
    echo "https://github.com/$REPO/releases/latest/download/$name"
  else
    echo "https://github.com/$REPO/releases/download/$VERSION/$name"
  fi
}

verify_checksum() {
  local binary_path="$1" asset_name="$2"
  local checksum_url checksum_tmp expected actual

  checksum_url=$(resolve_url "SHA256SUMS.txt")
  checksum_tmp=$(mktemp)

  info "$MSG_VERIFYING"
  if ! download "$checksum_url" "$checksum_tmp" 2>/dev/null; then
    rm -f "$checksum_tmp"
    warn "$MSG_CHECKSUM_SKIP_DL"
    return 0
  fi

  expected=$(grep "  ${asset_name}$" "$checksum_tmp" | awk '{print $1}')
  rm -f "$checksum_tmp"

  if [ -z "$expected" ]; then
    warn "$MSG_CHECKSUM_SKIP_MISSING"
    return 0
  fi

  if command -v sha256sum >/dev/null 2>&1; then
    actual=$(sha256sum "$binary_path" | awk '{print $1}')
  elif command -v shasum >/dev/null 2>&1; then
    actual=$(shasum -a 256 "$binary_path" | awk '{print $1}')
  else
    warn "$MSG_CHECKSUM_SKIP_CMD"
    return 0
  fi

  if [ "$expected" != "$actual" ]; then
    die "$(printf "$MSG_CHECKSUM_FAIL" "$expected" "$actual")"
  fi

  ok "$MSG_CHECKSUM_OK"
}

# ---------------------------------------------------------------------------
# 检查并自动安装 skm（samsara 的技能包管理器基础设施）
# ---------------------------------------------------------------------------
install_skm_if_needed() {
  local skm_bin
  if command -v skm >/dev/null 2>&1; then
    ok "$MSG_SKM_FOUND $(skm --version 2>&1 || true)"
    return 0
  fi
  skm_bin="$HOME/.local/bin/skm"
  if [ -x "$skm_bin" ]; then
    ok "$MSG_SKM_FOUND $("$skm_bin" --version 2>&1 || true)"
    return 0
  fi

  printf "\n"
  info "$MSG_SKM_INSTALLING"

  local skm_install_url="https://raw.githubusercontent.com/mocikadev/mocika-skills-cli/main/install.sh"
  local tmp_script
  tmp_script=$(mktemp)
  # shellcheck disable=SC2064
  trap "rm -f '$tmp_script'" EXIT

  if ! download "$skm_install_url" "$tmp_script" 2>/dev/null; then
    warn "$MSG_SKM_INSTALL_FAIL"
    return 0
  fi

  if bash "$tmp_script"; then
    ok "$MSG_SKM_INSTALL_OK"
  else
    warn "$MSG_SKM_INSTALL_FAIL"
  fi

  rm -f "$tmp_script"
}

main() {
  printf "\n${BOLD}%s${RESET}\n\n" "$MSG_TITLE"

  local target asset_name url tmp
  target=$(detect_target)
  asset_name="${BINARY}-${target}"
  url=$(resolve_url "$asset_name")

  info "$MSG_PLATFORM $target"
  info "$MSG_VERSION $VERSION"
  info "$MSG_INSTALL_DIR $INSTALL_DIR"
  printf "\n"

  mkdir -p "$INSTALL_DIR"

  info "$MSG_DOWNLOADING"
  tmp=$(mktemp)
  # shellcheck disable=SC2064
  trap "rm -f '$tmp'" EXIT

  download "$url" "$tmp" || die "$MSG_DOWNLOAD_FAIL"
  verify_checksum "$tmp" "$asset_name"
  chmod +x "$tmp"
  mv "$tmp" "$INSTALL_DIR/$BINARY"

  ok "$MSG_INSTALLED $INSTALL_DIR/$BINARY"

  if "$INSTALL_DIR/$BINARY" --version >/dev/null 2>&1; then
    ok "version: $("$INSTALL_DIR/$BINARY" --version 2>&1)"
  fi

  if ! echo ":${PATH}:" | grep -qF ":${INSTALL_DIR}:"; then
    printf "\n"
    warn "$(printf "$MSG_PATH_WARN" "$INSTALL_DIR")"
    printf "\n  ${BOLD}export PATH=\"\$HOME/.local/bin:\$PATH\"${RESET}\n"
  fi

  install_skm_if_needed

  printf "\n${GREEN}${BOLD}%s${RESET} $(printf "$MSG_HINT" "${BOLD}" "${RESET}")\n\n" "$MSG_DONE"
}

main "$@"
