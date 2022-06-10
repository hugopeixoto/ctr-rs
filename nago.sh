#!/usr/bin/env bash

for lang in ja ja-hrkt en fr it de es ko zh-hant zh-hans; do
  cargo run --bin pokedex-map -- pokemon-ultra-sun-eur.3ds $lang > nago/ultra-sun-$lang.csv
  cargo run --bin pokedex-map -- --alt pokemon-ultra-sun-eur.3ds $lang > nago/ultra-moon-$lang.csv

  cargo run --bin pokedex-map -- pokemon-moon-eur.3ds $lang > nago/moon-$lang.csv
  cargo run --bin pokedex-map -- --alt pokemon-moon-eur.3ds $lang > nago/sun-$lang.csv
done
