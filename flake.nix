{
  description = "Minimal Rust development environment for Bevy project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs; [
            http-server
            bun
            aseprite
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

                FLAKE_ROOT="$(git rev-parse --show-toplevel)"
                SRC="$FLAKE_ROOT/assets"
                DEST="$FLAKE_ROOT/src/assets"

                find "$SRC" -type f -name '*.aseprite' | while read -r file; do
                  rel="''${file#"$SRC"/}"
                  out="$DEST/''${rel%.aseprite}.png"

                  mkdir -p "$(dirname "$out")"

                  aseprite \
                    --batch "$file" \
                    --save-as "$out"
                done
              '';
            })
          ];
        };
      }
    );
}
