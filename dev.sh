trap 'kill $(jobs -p)' EXIT

npx live-server out/ &
watchexec -w src -w templates -w themes --restart -- \
  'for theme in minimalist retro contemporary; do cargo run -- 2026 --locale en-GB --theme themes/$theme.css > out/$theme.html.tmp && mv out/$theme.html.tmp out/$theme.html; done'
