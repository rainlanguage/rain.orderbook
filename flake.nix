{
  description = "Flake for development workflows.";

  inputs = {
    rainix.url = "github:rainprotocol/rainix/ad505fe1f10ac1ccb1cdbe028de24aeb73b193e5";
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
