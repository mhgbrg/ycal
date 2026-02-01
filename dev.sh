trap 'kill $(jobs -p)' EXIT

npx live-server out/ &
watchexec -w src -w templates -w config -w holidays --restart -- \
  'cargo run -- 2026 --config config/en.json --holidays holidays/england_and_wales_2026.json > out/en.html.tmp && mv out/en.html.tmp out/en.html'
