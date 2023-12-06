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
        rain-cli = "${rain.defaultPackage.${system}}/bin/rain";
        graphql-client = "${pkgs.graphql-client}/bin/graphql-client";
        jq = "${pkgs.jq}/bin/jq";

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
              ["OrderBook" "RainterpreterNPE2" "RainterpreterStoreNPE2" "RainterpreterParserNPE2" "RainterpreterExpressionDeployerNPE2" "ERC20Test"]
            )}
          '';

          init-setup =  pkgs.writeShellScriptBin "init-setup" (''
            forge build --root ../

            rm -rf ./abis ./tests/generated/*.json
            mkdir ./abis

            ${copy-subgraph-abis}
            ${copy-test-abis}
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

            # Point A
            start_time=$(date +"%s")

            sleep 1

            # Point B
            end_time=$(date +"%s")

            # Calculate time difference
            time_diff=$((end_time - start_time))
            echo "Time difference: $time_diff seconds"


            ${graphql-client} introspect-schema --output tests/utils/subgraph/wait/schema.json http://localhost:8030/graphql
            ${graphql-client} introspect-schema --output tests/utils/subgraph/query/schema.json http://localhost:8000/subgraphs/name/test/test

            # debug the content
            echo $(ls .)
            cat tests/utils/subgraph/query/schema.json | ${jq} .
            cat tests/utils/subgraph/wait/schema.json| ${jq} .
          '');

          default = rain_cli;
        };
      }
    );
}
