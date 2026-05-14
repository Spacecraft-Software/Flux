{
  description = "Flux — DNS Selector & Network Configurator";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "clippy" "rustfmt" ];
        };
        manifest = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = manifest.package.name;
          version = manifest.package.version;
          src = pkgs.lib.cleanSource ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          nativeBuildInputs = [ pkgs.installShellFiles ];
          buildFeatures = [ "tui" ];
          doCheck = false;
          postInstall = ''
            installManPage man/dns.1
          '';
          meta = with pkgs.lib; {
            description = manifest.package.description;
            homepage = manifest.package.homepage;
            license = licenses.gpl3Plus;
            maintainers = [ "Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>" ];
            mainProgram = "dns";
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            pkgs.cargo-deny
            pkgs.cargo-audit
            pkgs.jaq
            pkgs.dogdns
            pkgs.eza
            pkgs.bat
            pkgs.fd
            pkgs.procs
            pkgs.bottom
          ];
        };
      });
}
