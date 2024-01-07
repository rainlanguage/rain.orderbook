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
          deploy-single-contract = contract: ''
            forge script script/Deploy${contract}.sol:Deploy${contract} --legacy --verify --broadcast --rpc-url "''${CI_DEPLOY_RPC_URL}" --etherscan-api-key "''${EXPLORER_VERIFICATION_KEY}" \
              --sig='run(bytes)' \
              "$( ${(build-meta-cmd contract)} -E hex )" \
              ;
          '';

          deploy-contracts = rainix.mkTask.${system} { name = "deploy-contracts"; body = (''
            set -euo pipefail;
            forge build --force;
            echo 'deploy pubkey:'
            cast wallet address "''${DEPLOYMENT_KEY}";
          '' + pkgs.lib.concatStrings (map deploy-single-contract concrete-contracts)); };

          default = build-meta;
          ci-prep = build-meta;
        } // rainix.packages.${system};

        devShells = rainix.devShells.${system};
      }
    );

}
