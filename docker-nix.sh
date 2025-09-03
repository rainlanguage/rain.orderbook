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

# Configure git to trust the workspace directory and run nix with proper settings
docker run --rm \
  -v "$REPO_DIR:/workspace" \
  -w /workspace \
  --network host \
  $ENV_VARS \
  nixos/nix sh -c "
    git config --global --add safe.directory /workspace
    nix-env -iA nixpkgs.cacert || true
    export SSL_CERT_FILE=\$(find /nix/store -name ca-bundle.crt | head -1)
    export NIX_SSL_CERT_FILE=\$SSL_CERT_FILE
    export CURL_CA_BUNDLE=\$SSL_CERT_FILE
    exec nix --extra-experimental-features 'nix-command flakes' \"\$@\"
  " -- "$@"