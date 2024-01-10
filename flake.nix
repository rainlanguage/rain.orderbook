{
  description = "Flake for development workflows.";

  inputs = {
    rainix.url = "github:rainprotocol/rainix/6dcb9fa5ecae27914c3d0a2e19f3cd264b82a261";
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
