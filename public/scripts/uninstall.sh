#!/bin/sh
# gity - Uninstall Script

printf "\n\033[1;36m[*] Uninstalling gity...\033[0m\n"

DEST_DIRS="/usr/local/bin/gity $HOME/.local/bin/gity"
REMOVED=false

for dir in $DEST_DIRS; do
    if [ -f "$dir" ]; then
        rm "$dir"
        printf "[+] Removed $dir\n"
        REMOVED=true
    fi
done

if [ "$REMOVED" = false ]; then
    printf "[-] gity not found in common locations\n"
    printf "   Try: rm ~/.local/bin/gity\n"
fi

printf "\033[1;32m[+] Done\033[0m\n"
printf "SSH keys and config were left untouched\n"
printf "To remove: rm -rf ~/.ssh/id_ed25519_* ~/.config/gity\n"
printf "\n"