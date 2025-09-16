# a script to generate codegen files

# set -e

# LANG="$1"

# if [[ -z "$LANG" ]]; then
#   echo "Usage: $0 <typescript|rust|all>"
#   exit 1
# fi

# Generate IDL first
(cd schema && clear && cargo run --bin generate-idl)

# mkdir -p client/idl
# client/.crates/bin/shank idl --crate-root program --out-dir client/idl --out-filename solana_pinocchio_starter.json

# # Generate client
# cd client
# bun install
# bun run gen-client "$LANG"

# if [[ "$LANG" == "rust" || "$LANG" == "all" ]]; then
#   rustfmt rust/generated/*.rs rust/generated/**/*.rs
# fi
