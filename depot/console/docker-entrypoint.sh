#!/bin/sh
set -e

HTPASSWD_FILE="/etc/nginx/.htpasswd"

# Generate htpasswd file from environment variable
if [ -n "$CONSOLE_PASSWORD" ]; then
    echo "Setting up password authentication..."
    # Use the provided username or default to 'admin'
    USERNAME="${CONSOLE_USERNAME:-admin}"
    # Generate htpasswd file (using openssl since htpasswd may not be available)
    HASHED=$(openssl passwd -apr1 "$CONSOLE_PASSWORD")
    echo "${USERNAME}:${HASHED}" > "$HTPASSWD_FILE"
    echo "Authentication enabled for user: ${USERNAME}"
else
    echo "WARNING: No CONSOLE_PASSWORD set - authentication disabled!"
    echo "Set CONSOLE_PASSWORD environment variable to enable password protection."
    # Create a dummy htpasswd that allows any auth (won't be used since auth_basic is off)
    # Actually, we need to modify the config to disable auth when no password is set
    # For now, create an empty file so nginx doesn't error
    touch "$HTPASSWD_FILE"

    # Disable auth by modifying the config
    sed -i 's/auth_basic \$auth_basic_realm;/auth_basic off;/' /etc/nginx/conf.d/default.conf
fi

# Execute the original nginx command
exec "$@"
