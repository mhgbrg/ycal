gen *ARGS:
    cargo run --bin cli -- {{ARGS}}

holidays *ARGS:
    cargo run --bin holidays -- {{ARGS}}

wasm:
    wasm-pack build --target web

page: wasm
    mkdir -p docs
    cp static/* docs/
    cp pkg/*.js docs/
    cp pkg/*.d.ts docs/
    cp pkg/*.wasm docs/
    cp pkg/*.wasm.d.ts docs/

watch:
    watchexec --print-events -w src -w static -w templates -w themes --restart -- just page

serve:
    npx live-server docs/

[parallel]
dev-web: serve watch

dev-cli:
    npx live-server out/ &
    watchexec -w src -w static -w templates -w themes --restart -- \
        'for theme in minimalist retro contemporary; do cargo run --bin cli -- 2026 --locale en-GB --theme themes/$theme.css > out/$theme.html.tmp && mv out/$theme.html.tmp out/$theme.html; done'
