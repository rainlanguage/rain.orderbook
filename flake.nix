{
  description = "Flake for development workflows.";

  inputs = {
    rainix.url = "github:rainprotocol/rainix/ff671d13606dc12c096c9e2a70c8d26342bed5c1";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {self, flake-utils, rainix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = rainix.pkgs.${system};
      in {
        packages = rainix.packages.${system};
        devShells = rainix.devShells.${system};
      }
    );

}
