
nix develop ~/nix/vendor/external/gemini-cli/ -c bash -c "
  ~/nix/vendor/external/gemini-cli/bundle/gemini.js --output-format json \
			      --model gemini-2.5-flash
"
