set shell := ["zsh", "-uc"]

default:
    @just --list

run package *args='':
    @cargo run -p {{ package }} -- {{ args }}

build package *args='':
    @cargo build -p {{ package }} {{ args }}

run-server *args="": (run "ddns-server" args)

run-client *args="": (run "ddns-client" args)

build-server *args="--release": (build "ddns-server" args)

build-client *args="--release": (build "ddns-client" args)

clean:
    @cargo clean
