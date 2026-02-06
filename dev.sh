trap 'kill $(jobs -p)' EXIT

npx live-server out/ &
watchexec -w src -w templates -w config --restart -- \
  'for theme in minimalist retro contemporary; do cargo run -- 2026 --locale en-GB --holidays config/holidays/england_and_wales_2026.json --theme config/themes/$theme.css > out/$theme.html.tmp && mv out/$theme.html.tmp out/$theme.html; done'
