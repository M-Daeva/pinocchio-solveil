# The script to generate program ID and update it in files

clear

# Check if name parameter is provided
if [ -z "$1" ]; then
  echo "Usage: $0 <name>"
  exit 1
fi

NAME="$1"
NAME_UNDERSCORE="${NAME//-/_}"  # replace - with _

KEYPAIR_FILE="target/deploy/${NAME_UNDERSCORE}-keypair.json"

# 1) Generate keypair if it doesn't exist
if [ ! -f "$KEYPAIR_FILE" ]; then
  solana-keygen new --no-bip39-passphrase -o "$KEYPAIR_FILE"
  clear
fi

# 2) Get the pubkey
PUBKEY=$(solana --config ~/dev/.solana-configs/localnet.yml address -k "$KEYPAIR_FILE")

# 3) Replace declare_id! in lib.rs files
# Update programs directory
LIB_RS="./programs/${NAME}/src/lib.rs"
if [ -f "$LIB_RS" ]; then
  sed -i.bak -E "s#declare_id!\(\"[^\"]+\"\);#declare_id!(\"$PUBKEY\");#" "$LIB_RS"
  rm -f "${LIB_RS}.bak"
  echo "Updated: $LIB_RS"
else
  echo "File not found: $LIB_RS"
fi

# Update packages directory (look for packages that include the program name)
if [ -d "./packages" ]; then
  for package_dir in ./packages/*${NAME}*/; do
    if [ -d "$package_dir" ]; then
      PACKAGE_LIB_RS="${package_dir}src/lib.rs"
      if [ -f "$PACKAGE_LIB_RS" ]; then
        sed -i.bak -E "s#declare_id!\(\"[^\"]+\"\);#declare_id!(\"$PUBKEY\");#" "$PACKAGE_LIB_RS"
        rm -f "${PACKAGE_LIB_RS}.bak"
        echo "Updated: $PACKAGE_LIB_RS"
      fi
    fi
  done
fi

# 4) Replace pubkey in Anchor.toml for localnet and devnet
ANCHOR_TOML="./Anchor.toml"
if [ -f "$ANCHOR_TOML" ]; then
  sed -i.bak -E "/\[programs\.localnet\]/,/\[/{s#(${NAME}[[:space:]]*=[[:space:]]*\")[^\"]+(\".*)#\1$PUBKEY\2#}" "$ANCHOR_TOML"
  sed -i.bak -E "/\[programs\.devnet\]/,/\[/{s#(${NAME}[[:space:]]*=[[:space:]]*\")[^\"]+(\".*)#\1$PUBKEY\2#}" "$ANCHOR_TOML"
  rm -f "${ANCHOR_TOML}.bak"
else
  echo "File not found: $ANCHOR_TOML"
fi

echo "Updated pubkey: $PUBKEY"
