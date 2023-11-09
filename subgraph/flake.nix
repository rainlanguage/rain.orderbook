{
  description = "Flake for development orderbook subgraph workflows.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/ec750fd01963ab6b20ee1f0cb488754e8036d89d";
    flake-utils.url = "github:numtide/flake-utils";
  };


  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        jq = "${pkgs.jq}/bin/jq";

      in rec {
        packages = rec {
          concrete-contracts = ["OrderBook" "RainterpreterExpressionDeployerNP" "RainterpreterNP" "RainterpreterStore"];

          copy-abis = contract: ''
            cp ../out/${contract}.sol/${contract}.json ./test/generated/
          '';

          remove-duplicate-component = ''
            # Remove a component duplicated on RainterpreterExpressionDeployerNP abi that conflict with abigen
            contract_path="test/generated/RainterpreterExpressionDeployerNP.json"
            ${jq} '.abi |= map(select(.name != "StackUnderflow"))' $contract_path > updated_contract.json
            mv updated_contract.json $contract_path
          '';

          init-setup =  pkgs.writeShellScriptBin "init-setup" (''
            forge build --root ../

            rm -rf ./abis ./test/generated
            mkdir ./abis ./test/generated

            cp ../out/OrderBook.sol/OrderBook.json ./abis/
            cp ../out/ERC20.sol/ERC20.json ./abis/ERC20.json
            '' + pkgs.lib.concatStrings (map copy-abis concrete-contracts)
            + (remove-duplicate-component)
          );

          default = init-setup;
        };
      }
    );
}
