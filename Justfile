set shell := ["zsh", "-uc"]
migrate-config := "./crates/DDNS-Server/diesel.toml"
migrate-db-url := "./crates/DDNS-Server/db/data.db"
migrate-dir := "./crates/DDNS-Server/migrations"
migrate-options := "--config-file " + migrate-config + " --database-url " + migrate-db-url + " --migration-dir " + migrate-dir
export TAG := `git tag --sort=-creatordate | head -n 1 || echo "latest"`

@default:
    @just --list

@run package *args='':
    @cargo run -p {{ package }} -- {{ args }}

@build package *args='':
    @cargo build -p {{ package }} {{ args }}

@run-server *args="": (run "ddns-server" args)

@run-client *args="": (run "ddns-client" args)

@build-server *args="--release": (build "ddns-server" args)

@build-client *args="--release": (build "ddns-client" args)

@migration-list:
    diesel migration list {{ migrate-options }}

@migration-run:
    diesel migration run {{ migrate-options }}

@migration-revert *args:
    diesel migration revert {{ migrate-options }} {{ args }}

@migration-redo *args:
    diesel migration redo {{ migrate-options }} {{ args }}

@migration-generate *args:
    diesel migration generate {{ migrate-options }} {{ args }}

@clean:
    @cargo clean

@show-latest-tag:
    git tag --sort=-creatordate | head -n 1
@show-all-tags:
    git tag --sort=-creatordate

@add-tag +args:
    git tag {{args}} && git push origin {{args}}
@check-tags:
    @echo "Git Tag: {{TAG}}"
@remove-tag tag:
    @echo "正在刪除本地 Tag: {{tag}}..."
    git tag -d {{tag}}
    @echo "正在刪除遠端 Tag: {{tag}}..."
    git push origin --delete {{tag}}
    @echo "✅ Tag {{tag}} 已完全移除。"
