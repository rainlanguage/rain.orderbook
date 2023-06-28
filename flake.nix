{
  description = "Flake for development workflows.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rain.url = "github:rainprotocol/rain.cli";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {self, nixpkgs, rain, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        rain-cli = "${rain.defaultPackage.${system}}/bin/rain";

      in rec {
        packages = rec {
          contracts-with-meta = ["OrderBook" "GenericPoolOrderBookFlashBorrower"];
          build-single-meta = contract: ''
            ${rain-cli} meta build -o meta/${contract}.rain.meta \
              -i <(${rain-cli} meta solc artifact -c abi -i out/${contract}.sol/${contract}.json) -m solidity-abi-v2 -t json -e deflate -l en \
              -i src/concrete/${contract}.meta.json -m interpreter-caller-meta-v1 -t json -e deflate -l en \
              ;
          '';
          build-meta = pkgs.writeShellScriptBin "build-meta" (''
          set -x;
          forge build --force;
          '' + pkgs.lib.concatStrings (map build-single-meta contracts-with-meta));

          default = build-meta;
        };
      }
    );

}
