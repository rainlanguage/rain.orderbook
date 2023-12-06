{
  description = "Flake for development orderbook subgraph workflows.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/ec750fd01963ab6b20ee1f0cb488754e8036d89d";
    flake-utils.url = "github:numtide/flake-utils";
    rain.url = "github:rainlanguage/rain.cli/b702505ddd2a9bb714837a62b09d2ba08001c335";
  };


  outputs = { self, nixpkgs, rain, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        jq = "${pkgs.jq}/bin/jq";
        rain-cli = "${rain.defaultPackage.${system}}/bin/rain";
        graphql-client = "${pkgs.graphql-client}/bin/graphql-client";

      in rec {
        packages = rec {
          sg-abi-path = "./abis/";
          test-abi-path = "./tests/generated/";

          copy-abi = { origin_root, destiny, contract }: ''
            cp ${origin_root}/out/${contract}.sol/${contract}.json ${destiny}
          '';

          copy-subgraph-abis = ''
            # Copying contract ABIs needed for subgraph
            ${pkgs.lib.concatStrings (
              map (
                contract: copy-abi {
                  origin_root = "../";
                  destiny = sg-abi-path;
                  contract = contract; 
                })
              ["OrderBook" "ERC20"]
            )}
          '';

          copy-test-abis = ''
            # Copying contract ABIs needed for tests
            ${pkgs.lib.concatStrings (
              map (
                contract: copy-abi {
                  origin_root = "../";
                  destiny = test-abi-path;
                  contract = contract; 
                })
              ["OrderBook" "RainterpreterNP" "RainterpreterStore" "RainterpreterExpressionDeployerNP" "AuthoringMetaGetter" "ERC20Test"]
            )}
          '';

          remove-duplicate = ''
            # Remove a component duplicated on RainterpreterExpressionDeployerNP abi that conflict with abigen
            contract_path="tests/generated/RainterpreterExpressionDeployerNP.json"
            ${jq} '.abi |= map(select(.name != "StackUnderflow"))' $contract_path > updated_contract.json
            mv updated_contract.json $contract_path
          '';

          init-setup =  pkgs.writeShellScriptBin "init-setup" (''
            forge build --root ../

            rm -rf ./abis ./tests/generated/*.json
            mkdir ./abis

            ${copy-subgraph-abis}
            ${copy-test-abis}
            ${remove-duplicate}
          '');

          ci-test = pkgs.writeShellScriptBin "ci-test" (''
            cargo test -- --test-threads=1 --nocapture;
          '');

          build = pkgs.writeShellScriptBin  "build" (''
            ${rain-cli} subgraph build
          '');

          rain_cli = pkgs.writeShellScriptBin "rain_cli" (''
            ${rain-cli} $@
          '');

          docker-up = pkgs.writeShellScriptBin "docker-up" ''
            docker-compose -f docker/docker-compose.yaml up --build -d
          '';

          docker-down = pkgs.writeShellScriptBin "docker-down" ''
            docker-compose -f docker/docker-compose.yaml down
          '';

          generate-sg-schema =  pkgs.writeShellScriptBin "generate-sg-schema" (''
            ${rain-cli} subgraph build
            ${rain-cli} subgraph deploy --endpoint http://localhost:8020 --subgraph-name "test/test"

            # Wait for 1 second to the subgraph be totally deployed
            sleep 1

            ${graphql-client} introspect-schema --output tests/utils/subgraph/wait/schema.json http://localhost:8030/graphql
            ${graphql-client} introspect-schema --output tests/utils/subgraph/query/schema.json http://localhost:8000/subgraphs/name/test/test
          '');

          default = rain_cli;
        };
      }
    );
}
