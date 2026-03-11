{
  description = "DDNS-Remake workspace (Rust + contract)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    crane,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [
          rust-overlay.overlays.default
        ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        lib = pkgs.lib;
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        appleSdk =
          if pkgs ? apple-sdk
          then pkgs.apple-sdk
          else if pkgs ? apple-sdk_15
          then pkgs.apple-sdk_15
          else if pkgs ? apple-sdk_14
          then pkgs.apple-sdk_14
          else if pkgs ? apple-sdk_13
          then pkgs.apple-sdk_13
          else null;
        src = craneLib.cleanCargoSource ./.;
        commonNativeBuildInputs = with pkgs; [
          pkg-config
        ];
        commonBuildInputs = with pkgs;
          [
            openssl
            sqlite
          ]
          ++ lib.optionals pkgs.stdenv.isDarwin (
            lib.optional (appleSdk != null) appleSdk
          );
        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src;
          nativeBuildInputs = commonNativeBuildInputs;
          buildInputs = commonBuildInputs;
        };
        mkPkg = {
          pname,
          cargoExtraArgs ? "",
        }:
          craneLib.buildPackage {
            inherit pname src cargoArtifacts;
            version = "0.1.0";
            nativeBuildInputs = commonNativeBuildInputs;
            buildInputs = commonBuildInputs;
            inherit cargoExtraArgs;
            doCheck = true;
          };
        ddns-server = mkPkg {
          pname = "ddns-server";
          cargoExtraArgs = "-p ddns-server";
        };
        ddns-client = mkPkg {
          pname = "ddns-client";
          cargoExtraArgs = "-p ddns-client";
        };
      in {
        packages = {
          default = ddns-server;
          inherit ddns-server ddns-client;
        };
        apps = {
          ddns-server = flake-utils.lib.mkApp {drv = ddns-server;};
          ddns-client = flake-utils.lib.mkApp {drv = ddns-client;};
          just = {
            type = "app";
            program = "${pkgs.just}/bin/just";
          };
          default = flake-utils.lib.mkApp {drv = ddns-server;};
        };
        checks = {
          ddns-server = ddns-server;
          ddns-client = ddns-client;

          fmt = craneLib.cargoFmt {
            inherit src;
          };

          clippy = craneLib.cargoClippy {
            inherit src cargoArtifacts;
            nativeBuildInputs = commonNativeBuildInputs;
            buildInputs = commonBuildInputs;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          };

          test = craneLib.cargoTest {
            inherit src cargoArtifacts;
            nativeBuildInputs = commonNativeBuildInputs;
            buildInputs = commonBuildInputs;
          };
        };
        devShells.default = pkgs.mkShell {
          packages = with pkgs;
            [
              rustToolchain
              cargo-edit
              just
              git
              openssl
              sqlite
              pkg-config
              sqlx-cli
              nodejs_24
              pnpm
            ]
            ++ lib.optionals pkgs.stdenv.isDarwin [
              pkgs.libiconv
            ];
          shellHook = ''
            export RUST_BACKTRACE=1
            export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"
            export OPENSSL_DIR=${pkgs.openssl.dev}
            export OPENSSL_LIB_DIR=${pkgs.openssl.out}/lib
            export OPENSSL_INCLUDE_DIR=${pkgs.openssl.dev}/include
            echo "DevShell ready: rust=$(rustc --version), node=$(node --version), pnpm=$(pnpm --version)"
          '';
        };
      }
    );
}
