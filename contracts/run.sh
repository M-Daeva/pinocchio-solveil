#!/bin/bash

# ====================================================================================
# Solana Program Development Script
# ====================================================================================
# This script provides utilities for Solana program development including:
# - Program ID generation and updating
# - Building programs
# - Schema generation
# - Deploying programs
# - Running tests
#
# Usage Examples:
#   ./run.sh id registry        # Generate/update program ID for 'registry' program
#   ./run.sh build              # Build all programs and generate schema
#   ./run.sh build 0            # Build all programs only
#   ./run.sh build 1            # Generate schema only
#   ./run.sh deploy registry    # Deploy 'registry' program to devnet
#   ./run.sh test               # Build programs and run tests
#   ./run.sh test s             # Build programs and run tests with output
#   ./run.sh help               # Show this help message
# ====================================================================================

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

    deploy <name>   Deploy the specified program to devnet
                    Example: ./run.sh deploy registry

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
    
    deploy      - Deploys a specific program to devnet using its keypair
    
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
}

# Deploy a specific program to devnet
deploy() {
    clear
    # Check if name parameter is provided
    if [ -z "$1" ]; then
        echo "Usage: $0 deploy <name>"
        exit 1
    fi
    
    NAME="$1"
    NAME_UNDERSCORE="${NAME//-/_}"  # replace - with _
    KEYPAIR_FILE="target/deploy/${NAME_UNDERSCORE}-keypair.json"
    PROGRAM_FILE="target/deploy/${NAME_UNDERSCORE}.so"
    
    # Check if keypair file exists
    if [ ! -f "$KEYPAIR_FILE" ]; then
        echo "Error: Keypair file not found: $KEYPAIR_FILE"
        echo "Run './run.sh id $NAME' first to generate the program keypair"
        exit 1
    fi
    
    # Check if program file exists
    if [ ! -f "$PROGRAM_FILE" ]; then
        echo "Error: Program file not found: $PROGRAM_FILE"
        echo "Run './run.sh build' first to build the program"
        exit 1
    fi
    
    echo "Deploying $NAME to devnet..."
    echo "Keypair: $KEYPAIR_FILE"
    echo "Program: $PROGRAM_FILE"
    
    # Deploy the program to devnet
    solana --config ~/dev/.solana-configs/devnet.yml program deploy --program-id "$KEYPAIR_FILE" "$PROGRAM_FILE" --url devnet
    
    if [ $? -eq 0 ]; then
        echo "Successfully deployed $NAME to devnet!"
        PUBKEY=$(solana address -k "$KEYPAIR_FILE")
        echo "Program ID: $PUBKEY"
    else
        echo "Failed to deploy $NAME"
        exit 1
    fi
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
        "deploy")
            if [ -z "$2" ]; then
                echo "Error: Program name required for 'deploy' command"
                echo "Usage: ./run.sh deploy <program_name>"
                exit 1
            fi
            deploy "$2"
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
