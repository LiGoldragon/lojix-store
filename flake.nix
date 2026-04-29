{
  description = "arca — content-addressed filesystem (nix-store analogue; holds real unix files addressed by blake3). General-purpose; forge is one writer of many.";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  outputs = { self, nixpkgs }: let
    forAllSystems = nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
  in {
    devShells = forAllSystems (system: let pkgs = import nixpkgs { inherit system; }; in {
      default = pkgs.mkShell { packages = [ pkgs.jujutsu ]; };
    });
  };
}
