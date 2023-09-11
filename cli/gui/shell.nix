let
 pkgs = import <nixpkgs> {};

 dev = pkgs.writeShellScriptBin "dev" ''
  npm run dev
 '';

 mnemonic = pkgs.writeShellScriptBin "mnemonic" ''
  mnemonics
 '';

in
pkgs.stdenv.mkDerivation {
 name = "shell";
 buildInputs = [
  pkgs.nodejs-16_x
  dev
  mnemonic
 ];

 shellHook = ''
  source .env
  export PATH=$( npm bin ):$PATH
  git submodule update --init
  # keep it fresh
  npm install && npm run codegen
 '';
}