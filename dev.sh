trap 'kill $(jobs -p)' EXIT

npx live-server out/ &
watchexec -w src -w templates -w config --restart -- \
  'cargo run -- 2026 --locale config/locale/en.json --holidays config/holidays/england_and_wales_2026.json > out/en.html.tmp && mv out/en.html.tmp out/en.html'
