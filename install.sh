#!/bin/sh
# gity - Git Account Manager Installation Script
# Supports: Linux, macOS, Android (Termux)

set -e

show_help() {
    printf "gity installer - Manage multiple Git accounts safely\n\n"
    printf "Usage:\n"
    printf "  curl -fsSL gity.pages.dev/install | sh [OPTIONS]\n\n"
    printf "Options:\n"
    printf "  help      Show this help message\n\n"
    printf "Examples:\n"
    printf "  # Install gity\n"
    printf "  curl -fsSL gity.pages.dev/install | sh\n"
    exit 0
}

for arg in "$@"; do
    case "$arg" in
        help|--help|-h) show_help ;;
    esac
done

printf "\n\033[1;36m[*] Installing gity - Git Account Manager...\033[0m\n"
printf "\033[0;34m--------------------------------------------------\033[0m\n"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
  linux)
    case "$ARCH" in
      x86_64) TARGET_ARCH="x86_64"; PLATFORM="unknown-linux-gnu" ;;
      aarch64|arm64) TARGET_ARCH="aarch64"; PLATFORM="unknown-linux-musl" ;;
      *) printf "\n\033[0;31m[-] Error: Unsupported Linux Architecture: $ARCH\033[0m\n"; exit 1 ;;
    esac
    ;;
  darwin)
    PLATFORM="apple-darwin"
    case "$ARCH" in
      x86_64) TARGET_ARCH="x86_64" ;;
      aarch64|arm64) TARGET_ARCH="aarch64" ;;
      *) printf "\n\033[0;31m[-] Error: Unsupported macOS Architecture: $ARCH\033[0m\n"; exit 1 ;;
    esac
    ;;
  *)
    printf "\033[0;31m[-] Error: Unsupported OS: $OS\033[0m\n"
    exit 1
    ;;
esac

ASSET_NAME="gity-${TARGET_ARCH}-${PLATFORM}.tar.gz"
RELEASE_URL="https://api.github.com/repos/kristency/gity/releases/latest"

printf "[*] Fetching latest release...\n"
DOWNLOAD_URL=$(curl -s "$RELEASE_URL" | grep "browser_download_url" | grep "$ASSET_NAME" | cut -d '"' -f 4)

if [ -z "$DOWNLOAD_URL" ]; then
  printf "\033[0;31m[-] Error: Could not find asset $ASSET_NAME\033[0m\n"
  exit 1
fi

TMP_DIR=$(mktemp -d)
printf "[+] Downloading $ASSET_NAME...\n"
curl -sL "$DOWNLOAD_URL" -o "$TMP_DIR/gity.tar.gz"

printf "[*] Unpacking...\n"
tar -xzf "$TMP_DIR/gity.tar.gz" -C "$TMP_DIR"

DEST_DIR="/usr/local/bin"
if [ ! -w "$DEST_DIR" ]; then
    DEST_DIR="$HOME/.local/bin"
    mkdir -p "$DEST_DIR"
fi

cp "$TMP_DIR/gity" "$DEST_DIR/"
chmod +x "$DEST_DIR/gity"
printf "[+] Installed gity to $DEST_DIR\n"

rm -rf "$TMP_DIR"

printf "\033[1;32m[+] Success! gity installed\033[0m\n"
printf "\033[0;34m--------------------------------------------------\033[0m\n"

if echo "$PATH" | grep -q "$DEST_DIR"; then
    printf " * Add account:   \033[1mgity add\033[0m\n"
    printf " * List accounts: \033[1mgity list\033[0m\n"
    printf " * Clone repo:    \033[1mgity clone\033[0m\n"
    printf " * Security:      \033[1mgity audit\033[0m\n"
    printf " * Help:          \033[1mgity --help\033[0m\n"
else
    printf "\033[0;33m[!] Warning: $DEST_DIR not in PATH\033[0m\n"
    printf "Add to .bashrc/.zshrc: export PATH=\"\$PATH:$DEST_DIR\"\n"
fi

printf "\n * Uninstall: \033[1mcurl -fsSL gity.pages.dev/uninstall | sh\033[0m\n"
printf "\n"