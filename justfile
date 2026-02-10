gen *ARGS:
    cargo run --bin cli -- {{ARGS}}

holidays *ARGS:
    cargo run --bin holidays -- {{ARGS}}

wasm:
    wasm-pack build --target web

site: wasm
    mkdir -p site
    cp static/* site/
    cp pkg/*.js site/
    cp pkg/*.d.ts site/
    cp pkg/*.wasm site/
    cp pkg/*.wasm.d.ts site/

watch:
    watchexec --print-events -w src -w static -w templates -w themes --restart -- just site

serve:
    npx live-server site/

dev-web:
    just serve & just watch & wait

dev-cli:
    mkdir -p out/
    npx live-server out/ &
    watchexec -w src -w static -w templates -w themes --restart -- \
        'for theme in minimalist retro contemporary; do cargo run --bin cli -- 2026 --locale en-GB --theme themes/$theme.css > out/$theme.html.tmp && mv out/$theme.html.tmp out/$theme.html; done'
