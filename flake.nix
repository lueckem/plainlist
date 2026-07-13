{
  description = "plainlist";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";

      pkgs = import nixpkgs {
        inherit system;
      };

      manifest = builtins.fromTOML (builtins.readFile ./Cargo.toml);
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          rustc
          cargo
        ];
      };

      packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
        pname = manifest.package.name;
        version = manifest.package.version;
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;
      };
    };
}
