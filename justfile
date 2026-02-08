gen *ARGS:
    cargo run --bin cli -- {{ARGS}}

holidays *ARGS:
    cargo run --bin holidays -- {{ARGS}}

build:
    wasm-pack build --target web
    cp static/* pkg/

watch:
    watchexec --print-events -w src -w static -w templates -w themes --restart -- just build

serve:
    npx live-server pkg/

[parallel]
dev-web: serve watch

dev-cli:
    npx live-server out/ &
    watchexec -w src -w templates -w themes --restart -- \
        'for theme in minimalist retro contemporary; do cargo run --bin cli -- 2026 --locale en-GB --theme themes/$theme.css > out/$theme.html.tmp && mv out/$theme.html.tmp out/$theme.html; done'
