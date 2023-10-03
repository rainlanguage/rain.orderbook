# TODO: Improve tooling here
{
  description = "Flake for development orderbook subgraph workflows.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    subgraph-cli.url = "github:rainprotocol/rain.subgraph-cli";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {self, nixpkgs, subgraph-cli, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        rain-sg-cli = "${subgraph-cli.defaultPackage.${system}}/bin/rain_subgraph_cli";

      in rec {
        packages = rec {
          install = pkgs.writeShellScriptBin "install" (''${rain-sg-cli} install'');

          default = install;
        };
      }
    );

}
