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
        jq = "${pkgs.jq}/bin/jq";
        graphql_client = "${pkgs.graphql-client}/bin/graphql-client";

      in rec {
        packages = rec {
          install = pkgs.writeShellScriptBin "install" ("npm install");

          build = pkgs.writeShellScriptBin "build" ("npm run codegen && npm run build");

          # ERC20Mock is not present here. It's hardcoded because It's just a ERC20 contract with a mint method.
          concrete-contracts = ["AuthoringMetaGetter" "OrderBook" "RainterpreterExpressionDeployerNP" "RainterpreterNP" "RainterpreterStore"];

          copy-abis = contract: ''
            echo Copying ${contract}...
            cp ../out/${contract}.sol/${contract}.json ./tests/generated/

            # # Remove component duplicated that conflict with abigen
            # ${jq} '.abi |= map(select(.name != "StackUnderflow"))' contract.json > updated_contract.json
            # mv updated_contract.json tests/generated/RainterpreterExpressionDeployerNP.json
          '';

          remove-duplicate = ''
            # Remove a component duplicated on RainterpreterExpressionDeployerNP abi that 
            # conflict with abigen
            contract_path="tests/generated/RainterpreterExpressionDeployerNP.json"
            ${jq} '.abi |= map(select(.name != "StackUnderflow"))' $contract_path > updated_contract.json
            mv updated_contract.json $contract_path
            echo Removed duplicated at: $contract_path
          '';

          init-setup =  pkgs.writeShellScriptBin "init-setup" (''
            # NOTE: This should be called after `npm install`

            # Generating the contracts. This way, they will be updating by commit
            forge build --root ../
 
            # Copying the new abis into the SG abi folder
            cp ../out/OrderBook.sol/OrderBook.json ./abis/
            cp ../out/ERC20.sol/ERC20.json ./abis/ERC20.json
            '' + pkgs.lib.concatStrings (map copy-abis concrete-contracts) + (remove-duplicate)
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

          check = pkgs.writeShellScriptBin "check" (''
            ${graphql_client}
          '');

          strong-anvil = pkgs.writeShellScriptBin "strong-anvil" (''
            anvil -m "$(cat ./test-mnemonic)" --code-size-limit 36864
          '');

          end-anvil = pkgs.writeShellScriptBin "end-anvil" (''
            kill -9 $(lsof -t -i :8545)
          '');

          ci-test = pkgs.writeShellScriptBin "ci-test" (''
            # This build is for generate the schema.json
            cargo run build;
            ls
            cargo test -- --test-threads=1 --nocapture;
          '');

          default = install;
        };
      }
    );
}
