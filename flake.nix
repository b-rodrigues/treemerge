{
  description = "treemerge - merge text files from directory trees";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        rustPlatform = pkgs.rustPlatform;
      in {
        packages.default = rustPlatform.buildRustPackage {
          pname = "treemerge";
          version = "0.1.0";
          src = self;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = [ pkgs.pkg-config ];

          # Install the man page into share/man/man1
          postInstall = ''
            mkdir -p $out/share/man/man1
            cp ${self}/treemerge.1 $out/share/man/man1/
          '';
        };

        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/treemerge";
        };
      }
    );
}
