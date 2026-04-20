set shell := ["zsh", "-uc"]
migrate-config := "./crates/DDNS-Server/diesel.toml"
migrate-db-url := "./crates/DDNS-Server/db/data.db"
migrate-dir := "./crates/DDNS-Server/migrations"
migrate-options := "--config-file " + migrate-config + " --database-url " + migrate-db-url + " --migration-dir " + migrate-dir
export TAG := `git tag --sort=-creatordate | head -n 1 || echo "latest"`
export REGISTRY := "ghcr.io/derrick-program"

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

@build-image-server:
    nix build .#ddns-server-image
    @echo "✅ 鏡像編譯完成：./result"

@build-image-client:
    nix build .#ddns-client-image
    @echo "✅ 鏡像編譯完成：./result"

@load-server:
    @echo "🔨 Building Server Image via Nix..."
    nix build .#ddns-server-image
    @echo "🐋 Loading into Docker..."
    docker load < result
    @echo "🏷️ Tagging as {{TAG}}..."
    docker tag ddns-server:latest ddns-server:{{TAG}}

@load-client:
    @echo "🔨 Building Client Image via Nix..."
    nix build .#ddns-client-image
    @echo "🐋 Loading into Docker..."
    docker load < result
    @echo "🏷️ Tagging as {{TAG}}..."
    docker tag ddns-client:latest ddns-client:{{TAG}}

@docker-run-server: load-server
    docker run --rm -it ddns-server:{{TAG}}

@docker-run-client: load-client
    docker run --rm -it ddns-client:{{TAG}}

@push-server: load-server
    @echo "📤 推送 Server 鏡像到 GHCR..."
    docker tag ddns-server:{{TAG}} {{REGISTRY}}/ddns-server:{{TAG}}
    docker tag ddns-server:{{TAG}} {{REGISTRY}}/ddns-server:latest
    docker push {{REGISTRY}}/ddns-server:{{TAG}}
    docker push {{REGISTRY}}/ddns-server:latest
    @echo "✅ Server 鏡像已推送到 {{REGISTRY}}/ddns-server"

@push-client: load-client
    @echo "📤 推送 Client 鏡像到 GHCR..."
    docker tag ddns-client:{{TAG}} {{REGISTRY}}/ddns-client:{{TAG}}
    docker tag ddns-client:{{TAG}} {{REGISTRY}}/ddns-client:latest
    docker push {{REGISTRY}}/ddns-client:{{TAG}}
    docker push {{REGISTRY}}/ddns-client:latest
    @echo "✅ Client 鏡像已推送到 {{REGISTRY}}/ddns-client"