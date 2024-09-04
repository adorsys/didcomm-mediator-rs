#!/bin/bash

# Create .env.example file with test variables
cat <<EOL > .env.example
SERVER_PUBLIC_DOMAIN=https://example.com
SERVER_LOCAL_PORT=3000
STORAGE_DIRPATH="target/storage"
EOL

echo ".env.example file created successfully!"