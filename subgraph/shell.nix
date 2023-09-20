let
  pkgs = import
    (builtins.fetchTarball {
      name = "nixos-unstable-2021-10-01";
      url = "https://github.com/nixos/nixpkgs/archive/d3d2c44a26b693293e8c79da0c3e3227fc212882.tar.gz";
      sha256 = "0vi4r7sxzfdaxzlhpmdkvkn3fjg533fcwsy3yrcj5fiyqip2p3kl";
    })
    { };

  compile = pkgs.writeShellScriptBin "compile" ''
    hardhat compile --force
  '';

  codegen = pkgs.writeShellScriptBin "codegen" ''
    graph codegen
  '';

  docker-up = pkgs.writeShellScriptBin "docker-up" ''
    docker-down
    rm -rf docker/data
    docker-compose -f docker/docker-compose.yml up --build -d
  '';

  docker-down = pkgs.writeShellScriptBin "docker-down" ''
    docker-compose -f docker/docker-compose.yml down
  '';

  flush-all = pkgs.writeShellScriptBin "flush-all" ''
    rm -rf cache
    rm -rf node_modules
    rm -rf build
    rm -rf generated
    rm -rf docker/data
  '';

  copy-abis = pkgs.writeShellScriptBin "copy-abis" ''
    cp artifacts/contracts/orderbook/OrderBook.sol/OrderBook.json abis
    cp artifacts/contracts/test/testToken/ReserveToken.sol/ReserveToken.json abis
  '';

  setup = pkgs.writeShellScriptBin "setup" ''
    cp -r rain-protocol/artifacts .
    cp -r rain-protocol/typechain .
    copy-abis
  '';

  ci-test = pkgs.writeShellScriptBin "ci-test" ''
    npx mustache config/localhost.json subgraph.template.yaml subgraph.yaml
    codegen
    npx hardhat test --no-compile
  '';

  init = pkgs.writeShellScriptBin "init" ''
    npm install
  '';

  ci-prepare-subgraph-polygon = pkgs.writeShellScriptBin "ci-prepare-subgraph-polygon" ''
    npx mustache config/polygon.json subgraph.template.yaml subgraph.yaml
    graph codegen
    graph build
  '';

  ci-prepare-subgraph-mumbai = pkgs.writeShellScriptBin "ci-prepare-subgraph-mumbai" ''
    npx mustache config/mumbai.json subgraph.template.yaml subgraph.yaml
    graph codegen
    graph build
  '';
  
in
pkgs.stdenv.mkDerivation {
 name = "shell";
 buildInputs = [
  pkgs.nixpkgs-fmt
  pkgs.yarn
  pkgs.nodejs-16_x
  ci-test
  compile
  codegen
  docker-up
  docker-down
  flush-all
  init
  ci-prepare-subgraph-polygon
  ci-prepare-subgraph-mumbai
  copy-abis
  setup
 ];

 shellHook = ''
  export PATH=$( npm bin ):$PATH
 '';
}
