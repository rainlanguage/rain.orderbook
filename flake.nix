{
  description = "Flake for development workflows.";

  inputs = {
    rainix.url = "github:rainprotocol/rainix/49f820b308eda51ea5d1f9697e85012004f1cf05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {self, flake-utils, rainix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = rainix.pkgs.${system};

      in rec {
        packages = rec {
          concrete-contracts = ["OrderBook" "GenericPoolOrderBookV3FlashBorrower" "GenericPoolOrderBookV3ArbOrderTaker" "RouteProcessorOrderBookV3ArbOrderTaker"];
          build-meta-cmd = contract: ''
            rain meta build \
              -i <(rain meta solc artifact -c abi -i out/${contract}.sol/${contract}.json) -m solidity-abi-v2 -t json -e deflate -l en \
              -i src/concrete/${contract}.meta.json -m interpreter-caller-meta-v1 -t json -e deflate -l en \
          '';
          build-single-meta = contract: ''
            ${(build-meta-cmd contract)} -o meta/${contract}.rain.meta;
          '';

          build-meta = rainix.mkTask.${system} { name = "build-meta"; body = (''
          set -x;
          mkdir -p meta;
          forge build --force;
          '' + pkgs.lib.concatStrings (map build-single-meta concrete-contracts)); };

          deploy-single-contract = contract: ''
            forge script script/Deploy${contract}.sol:Deploy${contract} --legacy --verify --broadcast --rpc-url "''${CI_DEPLOY_RPC_URL}" --etherscan-api-key "''${EXPLORER_VERIFICATION_KEY}" \
              --sig='run(bytes)' \
              "$( ${(build-meta-cmd contract)} -E hex )" \
              ;
          '';
          deploy-contracts = pkgs.writeShellScriptBin "deploy-contracts" (''
            set -euo pipefail;
            forge build --force;
          '' + pkgs.lib.concatStrings (map deploy-single-contract concrete-contracts));

          default = build-meta;
          ci-prep = build-meta;
        } // rainix.packages.${system};

        devShells = rainix.devShells.${system};
      }
    );

}
