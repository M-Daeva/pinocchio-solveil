#!/bin/bash

# =============================================================================
# Solana Program Development Script
# =============================================================================
# This script provides utilities for Solana program development including:
# - Program ID generation and updating
# - Building programs
# - Schema generation
# - Running tests
#
# Usage Examples:
#   ./run.sh id registry    # Generate/update program ID for 'registry' program
#   ./run.sh build          # Build all programs and generate schema
#   ./run.sh build 0        # Build all programs only
#   ./run.sh build 1        # Generate schema only
#   ./run.sh test           # Build programs and run tests
#   ./run.sh test s         # Build programs and run tests with output
#   ./run.sh help           # Show this help message
# =============================================================================

# Display help information
help() {
    cat << EOF
Solana Program Development Script

USAGE:
    ./run.sh <command> [options]

COMMANDS:
    id <name>       Generate or update program ID for the specified program
                    Example: ./run.sh id registry

    build [flag]    Build programs and/or generate schema
                    No flag: Build all programs AND generate schema
                    0: Build all programs only
                    1: Generate schema only
                    Examples: 
                      ./run.sh build     # Build + schema
                      ./run.sh build 0   # Build only
                      ./run.sh build 1   # Schema only

    test [flag]     Build programs and run tests
                    No flag: Run tests normally
                    s: Run tests with detailed output (--show-output)
                    Examples:
                      ./run.sh test      # Build + test
                      ./run.sh test s    # Build + test with output

    help            Show this help message

DESCRIPTION:
    id <name>   - Generates a new keypair for the program if it doesn't exist
                - Updates declare_id! macros in lib.rs files
                - Supports both programs/ and packages/ directories
    
    build       - Compiles all Solana programs using cargo build-sbf
    
    schema      - Generates IDL schema files
    
    test        - Runs Cargo tests in the tests directory

EOF
}

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

# Build all Solana programs in the programs directory
build() {
    clear
    for program in programs/*/; do
        if [ -d "$program" ]; then
            echo "Building $(basename "$program")..."
            cd "$program" && cargo build-sbf && cd - > /dev/null
        fi
    done
}

# Generate IDL schema files
schema() {
    (cd schema && clear && cargo run --bin generate-idl)
    (cd ../scripts && codama run js)

    # mkdir -p client/idl
    # client/.crates/bin/shank idl --crate-root program --out-dir client/idl --out-filename solana_pinocchio_starter.json

    # # Generate client
    # cd client
    # bun install
    # bun run gen-client "$LANG"

    # if [[ "$LANG" == "rust" || "$LANG" == "all" ]]; then
    #   rustfmt rust/generated/*.rs rust/generated/**/*.rs
    # fi
}

# Run tests with optional show-output flag
test() {
    if [[ "$1" == "s" ]]; then
        (cd tests && clear && cargo test -- --show-output)
    else
        (cd tests && clear && cargo test)
    fi
}

# Main function to handle command routing
main() {
    case "$1" in
        "id")
            if [ -z "$2" ]; then
                echo "Error: Program name required for 'id' command"
                echo "Usage: ./run.sh id <program_name>"
                exit 1
            fi
            update_id "$2"
            ;;
        "build")
            case "$2" in
                "0")
                    # Build only
                    build
                    ;;
                "1")
                    # Schema only
                    schema
                    ;;
                "")
                    # Default: build and schema
                    build
                    schema
                    ;;
                *)
                    echo "Error: Invalid build flag '$2'"
                    echo "Valid options: 0 (build only), 1 (schema only), or no flag (both)"
                    exit 1
                    ;;
            esac
            ;;
        "test")
            # Always build before testing
            build
            case "$2" in
                "s")
                    test "s"
                    ;;
                "")
                    test
                    ;;
                *)
                    echo "Error: Invalid test flag '$2'"
                    echo "Valid options: s (show output) or no flag (normal)"
                    exit 1
                    ;;
            esac
            ;;
        "help"|"-h"|"--help")
            help
            ;;
        "")
            echo "Error: No command specified"
            echo "Run './run.sh help' for usage information"
            exit 1
            ;;
        *)
            echo "Error: Unknown command '$1'"
            echo "Run './run.sh help' for available commands"
            exit 1
            ;;
    esac
}

# Execute main function with all arguments
main "$@"


























# # Consolidated run.sh script
# # Usage: ./run.sh <command> [flags]
# # Example: ./run.sh test s

# set -e  # Exit on any error

# # possible options
# # ./run.sh id registry  # update_id() registry
# # ./run.sh build        # build() && schema()
# # ./run.sh build 0      # build()
# # ./run.sh build 1      # schema()
# # ./run.sh test         # build() && test()
# # ./run.sh test s       # build() && test() s

# # The script to generate program ID and update it in files
# # Example: ./update_id.sh registry
# update_id() {
#     clear

#     # Check if name parameter is provided
#     if [ -z "$1" ]; then
#         echo "Usage: $0 <name>"
#         exit 1
#     fi

#     NAME="$1"
#     NAME_UNDERSCORE="${NAME//-/_}"  # replace - with _

#     KEYPAIR_FILE="target/deploy/${NAME_UNDERSCORE}-keypair.json"

#     # 1) Generate keypair if it doesn't exist
#     if [ ! -f "$KEYPAIR_FILE" ]; then
#         solana-keygen new --no-bip39-passphrase -o "$KEYPAIR_FILE"
#         clear
#     fi

#     # 2) Get the pubkey
#     PUBKEY=$(solana --config ~/dev/.solana-configs/localnet.yml address -k "$KEYPAIR_FILE")

#     # 3) Replace declare_id! in lib.rs files
#     # Update programs directory
#     LIB_RS="./programs/${NAME}/src/lib.rs"
#     if [ -f "$LIB_RS" ]; then
#         sed -i.bak -E "s#declare_id!\(\"[^\"]+\"\);#declare_id!(\"$PUBKEY\");#" "$LIB_RS"
#         rm -f "${LIB_RS}.bak"
#         echo "Updated: $LIB_RS"
#     else
#         echo "File not found: $LIB_RS"
#     fi

#     # Update packages directory (look for packages that include the program name)
#     if [ -d "./packages" ]; then
#         for package_dir in ./packages/*${NAME}*/; do
#             if [ -d "$package_dir" ]; then
#                 PACKAGE_LIB_RS="${package_dir}src/lib.rs"
#                 if [ -f "$PACKAGE_LIB_RS" ]; then
#                     sed -i.bak -E "s#declare_id!\(\"[^\"]+\"\);#declare_id!(\"$PUBKEY\");#" "$PACKAGE_LIB_RS"
#                     rm -f "${PACKAGE_LIB_RS}.bak"
#                     echo "Updated: $PACKAGE_LIB_RS"
#                 fi
#             fi
#         done
#     fi

#     # # 4) Replace pubkey in Anchor.toml for localnet and devnet
#     # ANCHOR_TOML="./Anchor.toml"
#     # if [ -f "$ANCHOR_TOML" ]; then
#     #     sed -i.bak -E "/\[programs\.localnet\]/,/\[/{s#(${NAME}[[:space:]]*=[[:space:]]*\")[^\"]+(\".*)#\1$PUBKEY\2#}" "$ANCHOR_TOML"
#     #     sed -i.bak -E "/\[programs\.devnet\]/,/\[/{s#(${NAME}[[:space:]]*=[[:space:]]*\")[^\"]+(\".*)#\1$PUBKEY\2#}" "$ANCHOR_TOML"
#     #     rm -f "${ANCHOR_TOML}.bak"
#     # else
#     #     echo "File not found: $ANCHOR_TOML"
#     # fi

#     echo "Updated pubkey: $PUBKEY"
# }


# build() {
#     clear

#     for program in programs/*/; do
#         if [ -d "$program" ]; then
#             echo "Building $(basename "$program")..."
#             cd "$program" && cargo build-sbf && cd - > /dev/null
#         fi
#     done
# }

# schema() {
#     (cd schema && clear && cargo run --bin generate-idl)
# }

# test() {
#   if [[ "$1" == "s" ]]; then
#       (cd tests && clear && cargo test -- --show-output)
#   else
#       (cd tests && clear && cargo test)
#   fi
# }




# # Help function
# show_help() {
#     echo "Usage: $0 <command> [flags]"
#     echo "Commands:"
#     echo "  build     - Run build + codegen"
#     echo "  test      - Run build + test (with optional flags)"
#     echo "  codegen   - Run codegen only"
#     echo "Flags:"
#     echo "  s         - Special argument for test command"
#     echo "Examples:"
#     echo "  $0 build           # runs build() + codegen()"
#     echo "  $0 test            # runs build() + test('')"
#     echo "  $0 test s          # runs build() + test('s')"
#     echo "  $0 codegen         # runs codegen() only"
# }

# # Main script logic
# main() {
#     local command="$1"
#     local flags="$2"
    
#     case "$command" in
#         "build")
#             build "$flags"
#             ;;
#         "test")
#             test_func "$flags"
#             ;;
#         "codegen")
#             codegen "$flags"
#             ;;
#         "help"|"-h"|"--help")
#             show_help
#             ;;
#         "")
#             echo "Error: No command provided"
#             show_help
#             exit 1
#             ;;
#         *)
#             echo "Error: Unknown command '$command'"
#             show_help
#             exit 1
#             ;;
#     esac
# }

# # Call main function with all arguments
# main "$@"