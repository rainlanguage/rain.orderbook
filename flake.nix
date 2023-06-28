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
          contracts-with-meta = ["CloneFactory" "GenericPoolOrderBookFlashBorrower"];
          build-single-meta = contract: ''
            ${rain-cli} meta build -o meta/${contract}.rain.meta \
              -i <(${rain-cli} meta solc artifact -c abi -i out/${contract}.sol/${contract}.json) -m solidity-abi-v2 -t json -e deflate -l en \
              -i src/concrete/${contract}.meta.json -m interpreter-caller-meta-v1 -t json -e deflate -l en \
              ;
          '';
          forge-build = ''
          forge build;
          '';
          build-meta = pkgs.writeShellScriptBin "build-meta" builtIns.concatStrings [forge-build builtIns.concatStrings map(build-single-meta packages.contracts-with-meta)];

          default = build-meta;
        };
      }
    );

}
