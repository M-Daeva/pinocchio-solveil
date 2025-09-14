#!/bin/bash

# Consolidated run.sh script
# Usage: ./run.sh <command> [flags]
# Example: ./run.sh test s

set -e  # Exit on any error

# possible options
# ./run.sh id registry  # update_id() registry
# ./run.sh build        # build() && codegen()
# ./run.sh build 0      # build()
# ./run.sh build 1      # codegen()
# ./run.sh test         # build() && test()
# ./run.sh test s       # build() && test() s

# The script to generate program ID and update it in files
# Example: ./update_id.sh registry
update_id() {
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

    # # 4) Replace pubkey in Anchor.toml for localnet and devnet
    # ANCHOR_TOML="./Anchor.toml"
    # if [ -f "$ANCHOR_TOML" ]; then
    #     sed -i.bak -E "/\[programs\.localnet\]/,/\[/{s#(${NAME}[[:space:]]*=[[:space:]]*\")[^\"]+(\".*)#\1$PUBKEY\2#}" "$ANCHOR_TOML"
    #     sed -i.bak -E "/\[programs\.devnet\]/,/\[/{s#(${NAME}[[:space:]]*=[[:space:]]*\")[^\"]+(\".*)#\1$PUBKEY\2#}" "$ANCHOR_TOML"
    #     rm -f "${ANCHOR_TOML}.bak"
    # else
    #     echo "File not found: $ANCHOR_TOML"
    # fi

    echo "Updated pubkey: $PUBKEY"
}


build() {
    clear

    for program in programs/*/; do
        if [ -d "$program" ]; then
            echo "Building $(basename "$program")..."
            cd "$program" && cargo build-sbf && cd - > /dev/null
        fi
    done
}

test_func() {
  if [[ "$1" == "s" ]]; then
      (cd tests && clear && cargo test -- --show-output)
  else
      (cd tests && clear && cargo test)
  fi
}

codegen() {
    local flags="$1"
    echo "Running codegen function with flags: $flags"
    
    # Your codegen.sh content goes here
    # Example:
    # if [[ "$flags" == "s" ]]; then
    #     echo "Generating code with special options"
    # else
    #     echo "Standard code generation"
    # fi
}


# Help function
show_help() {
    echo "Usage: $0 <command> [flags]"
    echo "Commands:"
    echo "  build     - Run build + codegen"
    echo "  test      - Run build + test (with optional flags)"
    echo "  codegen   - Run codegen only"
    echo "Flags:"
    echo "  s         - Special argument for test command"
    echo "Examples:"
    echo "  $0 build           # runs build() + codegen()"
    echo "  $0 test            # runs build() + test('')"
    echo "  $0 test s          # runs build() + test('s')"
    echo "  $0 codegen         # runs codegen() only"
}

# Main script logic
main() {
    local command="$1"
    local flags="$2"
    
    case "$command" in
        "build")
            build "$flags"
            ;;
        "test")
            test_func "$flags"
            ;;
        "codegen")
            codegen "$flags"
            ;;
        "help"|"-h"|"--help")
            show_help
            ;;
        "")
            echo "Error: No command provided"
            show_help
            exit 1
            ;;
        *)
            echo "Error: Unknown command '$command'"
            show_help
            exit 1
            ;;
    esac
}

# Call main function with all arguments
main "$@"