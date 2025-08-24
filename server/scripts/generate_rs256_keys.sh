#!/bin/bash

# Script to generate RS256 key pair for JWT signing
# This creates both private and public keys required for RS256 algorithm

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Generating RS256 key pair for JWT signing...${NC}"

# Define key directory and file names
KEY_DIR="core/keys"
PRIVATE_KEY="$KEY_DIR/jwt_private_key.pem"
PUBLIC_KEY="$KEY_DIR/jwt_public_key.pem"

# Create keys directory if it doesn't exist
if [ ! -d "$KEY_DIR" ]; then
    echo -e "${YELLOW}Creating keys directory: $KEY_DIR${NC}"
    mkdir -p "$KEY_DIR"
fi

# Generate private key (2048 bits RSA)
echo -e "${GREEN}Generating private key...${NC}"
openssl genpkey -algorithm RSA -out "$PRIVATE_KEY" -pkeyopt rsa_keygen_bits:2048

# Extract public key from private key
echo -e "${GREEN}Extracting public key...${NC}"
openssl rsa -pubout -in "$PRIVATE_KEY" -out "$PUBLIC_KEY"

# Set appropriate permissions (private key should be readable only by owner)
chmod 600 "$PRIVATE_KEY"
chmod 644 "$PUBLIC_KEY"

echo -e "${GREEN}✓ RS256 key pair generated successfully!${NC}"
echo -e "${GREEN}Private key: $PRIVATE_KEY${NC}"
echo -e "${GREEN}Public key: $PUBLIC_KEY${NC}"

# Add keys directory to .gitignore if not already there
if ! grep -q "core/keys/" .gitignore 2>/dev/null; then
    echo -e "${YELLOW}Adding keys directory to .gitignore...${NC}"
    echo -e "\n# JWT RS256 keys (never commit these!)" >> .gitignore
    echo "core/keys/" >> .gitignore
    echo -e "${GREEN}✓ Added keys directory to .gitignore${NC}"
fi

echo -e "${YELLOW}⚠️  WARNING: Never commit private keys to version control!${NC}"
echo -e "${YELLOW}For production, store keys securely using environment variables or secret management systems.${NC}"