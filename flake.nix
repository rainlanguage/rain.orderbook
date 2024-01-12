{
  description = "Flake for development workflows.";

  inputs = {
    rainix.url = "github:rainprotocol/rainix/2573ca453152200d1b906a2e8cc1cd0d0c73be4c";
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
