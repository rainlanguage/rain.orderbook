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


          gen-sg-schema = ''
            # Use a arbitrary address to put the endpoint up
            cargo run deploy \
              --name test/test \
              --url http://localhost:8020 \
              --network localhost \
              --block 0 \
              --address 0x0000000000000000000000000000000000000000

           ${graphql_client} introspect-schema \
            --output tests/subgraph/query/schema.json \
            http://localhost:8000/subgraphs/name/test/test
          '';

          init-setup =  pkgs.writeShellScriptBin "init-setup" (''
            # NOTE: This should be called after `npm install`

            # Generating the contracts. This way, they will be updating by commit
            forge build --root ../
 
            # Copying the new abis into the SG abi folder
            cp ../out/OrderBook.sol/OrderBook.json ./abis/
            cp ../out/ERC20.sol/ERC20.json ./abis/ERC20.json
            '' + pkgs.lib.concatStrings (map copy-abis concrete-contracts)
            + (remove-duplicate)
          );

          run-anvil = pkgs.writeShellScriptBin "run-anvil" (''
            anvil -m "$(cat ./test-mnemonic)"
          '');

          docker-up = pkgs.writeShellScriptBin "docker-up" ''
            docker-compose -f docker/docker-compose.yaml up --build -d
          '';

          docker-down = pkgs.writeShellScriptBin "docker-down" ''
            docker-compose -f docker/docker-compose.yaml down
          '';

          end-anvil = pkgs.writeShellScriptBin "end-anvil" (''
            kill -9 $(lsof -t -i :8545)
          '');

          # The graphql file can generate the schema.json file needed for testing
          # Of course, this need a graph node at localhost to work
          gen-subgraph-schema  = pkgs.writeShellScriptBin "gen-subgraph-schema" (''
            # Use a arbitrary address to put the endpoint up
            cargo run deploy \
              --name test/test \
              --url http://localhost:8020 \
              --network localhost \
              --block 0 \
              --address 0x0000000000000000000000000000000000000000

           ${graphql_client} introspect-schema \
            --output tests/subgraph/query/schema.json \
            http://localhost:8000/subgraphs/name/test/test
          '');

          ci-test = pkgs.writeShellScriptBin "ci-test" (''

            # Run tests in single thread
            cargo test -- --test-threads=1 --nocapture;
          '');

          default = init-setup;
        };
      }
    );
}
