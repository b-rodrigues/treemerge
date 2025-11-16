{
  description = "treemerge - merge text files from directory trees";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachSystem [
      "x86_64-linux"
      "aarch64-linux"
      "x86_64-darwin"
      "aarch64-darwin"
    ]
    (system:
      let
        pkgs = import nixpkgs { inherit system; };
        rustPlatform = pkgs.rustPlatform;
      in {
        packages.treemerge = rustPlatform.buildRustPackage {
          pname = "treemerge";
          version = "0.0.3";
          src = self;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = [ pkgs.pkg-config ];

          postInstall = ''
            mkdir -p $out/share/man/man1
            cp ${self}/treemerge.1 $out/share/man/man1/
          '';
        };

        # Make `nix build` use the native target on this system
        packages.default = self.packages.${system}.treemerge;

        apps.default = {
          type = "app";
          program = "${self.packages.${system}.treemerge}/bin/treemerge";
        };
      }
    );
}
