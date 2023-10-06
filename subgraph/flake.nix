# TODO: Improve tooling here
{
  description = "Flake for development orderbook subgraph workflows.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

      in rec {
        packages = rec {
          install = pkgs.writeShellScriptBin "install" ("npm install");

          build = pkgs.writeShellScriptBin "build" ("npm run codegen && npm run build");

          init-setup =  pkgs.writeShellScriptBin "init-setup" (''
            # NOTE: This should be called after `npm install`

            # Generating the contracts. This way, they will be updating by commit
            forge build --root ../

            # Copying the new abis into the SG abi folder
            cp ../out/OrderBook.sol/OrderBook.json ./abis/
            cp ../out/ERC20.sol/ERC20.json ./abis/ReserveToken.json            
            ''
          );

          docker-up = pkgs.writeShellScriptBin "docker-up" ''
            docker-compose -f docker/docker-compose.yaml up --build -d
          '';

          docker-down = pkgs.writeShellScriptBin "docker-down" ''
            docker-compose -f docker/docker-compose.yaml down
          '';


          check-args = pkgs.writeShellScriptBin "check-args" (''
            echo "All parameters: $@"
            echo "First parameter: $1"
            echo "Second parameter: $2"
          '');

          run-anvil = pkgs.writeShellScriptBin "run-anvil" (''
            anvil -m "$(cat ./test-mnemonic)"
          '');

          end-anvil = pkgs.writeShellScriptBin "end-anvil" (''
            kill -9 $(lsof -t -i :8545)
          '');

          ci-test = pkgs.writeShellScriptBin "ci-test" (''
            clear;
            cargo test -- --nocapture;
            kill -9 $(lsof -t -i :8545);
          '');

          

          default = install;

        };
      }
    );
}
