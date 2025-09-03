#!/bin/bash

# Wrapper script to run nix commands in Docker
# This solves the issue where nix is not installed on the host

set -e

# Get the absolute path of the current directory
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Pass through environment variables that the nix scripts need
ENV_VARS=""
for var in CI_DEPLOY_SEPOLIA_RPC_URL CI_FORK_SEPOLIA_DEPLOYER_ADDRESS CI_FORK_SEPOLIA_BLOCK_NUMBER CI_DEPLOY_POLYGON_RPC_URL CI_SEPOLIA_METABOARD_URL RPC_URL_ETHEREUM_FORK COMMIT_SHA PUBLIC_WALLETCONNECT_PROJECT_ID; do
    if [ ! -z "${!var}" ]; then
        ENV_VARS="$ENV_VARS -e $var=${!var}"
    fi
done

# Run nix in Docker with proper configuration
docker run --rm \
  -v "$REPO_DIR:/workspace" \
  -w /workspace \
  --network host \
  $ENV_VARS \
  nixos/nix sh -c "
    # Trust the workspace directory
    git config --global --add safe.directory /workspace
    
    # Configure git to handle SSL issues
    git config --global http.sslverify false
    
    # Configure SSL certificate handling
    export NIX_SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
    export CURL_CA_BUNDLE=/etc/ssl/certs/ca-certificates.crt
    export SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
    
    # Run nix command
    exec nix --extra-experimental-features 'nix-command flakes' \"\$@\"
  " -- "$@"