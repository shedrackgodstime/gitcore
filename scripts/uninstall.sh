#!/bin/sh
# gity - Uninstall Script

printf "\n\033[1;36m[*] Uninstalling gity...\033[0m\n"

# Check common installation paths
DEST_PATHS="/usr/local/bin/gity $HOME/.local/bin/gity"
REMOVED=false

for path in $DEST_PATHS; do
    if [ -f "$path" ]; then
        rm "$path"
        printf "[+] Removed $path\n"
        REMOVED=true
    fi
done

if [ "$REMOVED" = false ]; then
    printf "[-] gity not found in common locations\n"
else
    printf "\033[1;32m[+] Done\033[0m\n"
fi

printf "\n"
printf "SSH keys and gity configuration were left untouched for safety.\n"
printf "To remove them manually:\n"
printf "  rm -rf ~/.config/gity\n"
printf "  rm ~/.ssh/id_ed25519_* (Careful!)\n"
printf "\n"
