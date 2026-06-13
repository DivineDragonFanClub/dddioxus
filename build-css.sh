#!/bin/sh
# regenerate the tailwind stylesheet the desktop app loads via asset!.
# run this after adding or changing any tailwind class names in src/.
cd "$(dirname "$0")"
./tailwindcss -i tailwind.input.css -o assets/tailwind.css --minify "$@"
