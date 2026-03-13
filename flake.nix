{
  description = "DDNS-Remake Workspace";

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
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };
        inherit (pkgs) lib;

        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        appleSdk = with pkgs;
          lib.findFirst (sdk: sdk ? out) null
          (builtins.filter (sdk: sdk != null) [
            apple-sdk_15
            apple-sdk_14
            apple-sdk
          ]);

        commonNativeBuildInputs = with pkgs; [
          pkg-config
          clang
          # mold
          stdenv.cc
        ];

        commonBuildInputs = with pkgs;
          [
            openssl
            sqlite
          ]
          ++ lib.optionals stdenv.isDarwin (
            [libiconv] ++ lib.optional (appleSdk != null) appleSdk
          );

        commonEnv = {
          CC = "${pkgs.stdenv.cc}/bin/cc";
          CXX = "${pkgs.stdenv.cc}/bin/c++";
          OPENSSL_DIR = pkgs.openssl.dev;
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
          # RUSTFLAGS = "-C link-arg=-fuse-ld=mold";
        };
        src = craneLib.cleanCargoSource (craneLib.path ./.);
        cargoArtifacts = craneLib.buildDepsOnly (commonEnv
          // {
            inherit src;
            nativeBuildInputs = commonNativeBuildInputs;
            buildInputs = commonBuildInputs;
          });
        mkPkg = {
          pname,
          cargoExtraArgs ? "",
        }:
          craneLib.buildPackage (commonEnv
            // {
              inherit pname src cargoArtifacts cargoExtraArgs;
              version = "0.1.0";
              nativeBuildInputs = commonNativeBuildInputs;
              buildInputs = commonBuildInputs;
              doCheck = true;
            });

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

        apps = rec {
          ddns-server = flake-utils.lib.mkApp {drv = self.packages.${system}.ddns-server;};
          ddns-client = flake-utils.lib.mkApp {drv = self.packages.${system}.ddns-client;};
          just = { type = "app"; program = "${pkgs.just}/bin/just"; };
          default = ddns-server;
        };
        devShells.default = pkgs.mkShell {
          inputsFrom = [cargoArtifacts];

          packages = with pkgs; [
            rustToolchain
            cargo-edit
            just
            git
            sqlx-cli
            (diesel-cli.override {
              sqliteSupport = true;
              postgresqlSupport = false;
              mysqlSupport = false;
            })
            nodejs_24
            pnpm
            sccache
            mold
            clang
            cargo-sort
          ];
          # inherit (commonEnv) CC CXX OPENSSL_DIR OPENSSL_LIB_DIR OPENSSL_INCLUDE_DIR RUSTFLAGS;
          inherit (commonEnv) CC CXX OPENSSL_DIR OPENSSL_LIB_DIR OPENSSL_INCLUDE_DIR;
          shellHook = ''
            export RUST_BACKTRACE=1
            export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"
            export RUSTC_WRAPPER=sccache
            export SCCACHE_DIR="$PWD/.cache/sccache"
            export PATH="${pkgs.stdenv.cc}/bin:$PATH"

            echo "🚀 DDNS-Remake 開發環境已就緒"
            echo "編譯器: $(cc --version | head -n 1)"
            echo "Rust: $(rustc --version)"
            echo "Diesel CLI: $(diesel --version)"
          '';
        };
      }
    );
}
