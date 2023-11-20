{
  description = "Flake for development orderbook subgraph workflows.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/ec750fd01963ab6b20ee1f0cb488754e8036d89d";
    flake-utils.url = "github:numtide/flake-utils";
    rain.url = "github:rainprotocol/rain.cli/9f5d4bf65bee767f80871d9952665c654b01bdfb";
  };


  outputs = { self, nixpkgs, rain, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        jq = "${pkgs.jq}/bin/jq";
        rain-cli = "${rain.defaultPackage.${system}}/bin/rain";

      in rec {
        packages = rec {
          init-setup =  pkgs.writeShellScriptBin "init-setup" (''echo init'');

          shell = pkgs.mkShell {
            buildInputs = [ rain-cli ];
          };

          default = init-setup;
        };
      }
    );
}
