gen *ARGS:
    cargo run --bin cli -- {{ARGS}}

serve *ARGS:
    cargo run --bin server -- {{ARGS}}

dev-cli:
    npx live-server out/ &
    watchexec -w src -w templates -w themes --restart -- \
        'for theme in minimalist retro contemporary; do cargo run --bin cli -- 2026 --locale en-GB --theme themes/$theme.css > out/$theme.html.tmp && mv out/$theme.html.tmp out/$theme.html; done'

dev-server:
    watchexec -w src -w templates -w themes --restart -- cargo run --bin server
