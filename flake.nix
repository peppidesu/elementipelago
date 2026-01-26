{
  description = "Minimal Rust development environment for Bevy project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    png-subs.url = "https://git.geenit.nl/noa/png-subs/archive/main.tar.gz";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      png-subs,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };
        png-sub = png-subs.packages.${system}.default;
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs; [
            http-server
            bun
            aseprite
            png-sub
            (pkgs.writeShellApplication {
              name = "aseprite-export";

              runtimeInputs = [
                pkgs.aseprite
                pkgs.git
                pkgs.findutils
                pkgs.coreutils
              ];

              text = ''
                set -euo pipefail
                ./build-sprites.sh
              '';
            })
          ];
        };
      }
    );
}
