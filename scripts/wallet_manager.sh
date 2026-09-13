#!/usr/bin/env bash
# Market Earth AMM CLI Wallet Manager Helper
set -eo pipefail

export PATH="$HOME/.local/share/dfx/bin:$PATH"

echo "=============================================="
echo "🌍 Market Earth AMM CLI Wallet Manager 🌍"
echo "=============================================="

# Helper function to print usage
print_usage() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  status               Show current active identity, principal, and balances"
    echo "  use <name>           Switch active identity to <name>"
    echo "  create <name>        Create a new identity named <name>"
    echo "  import <name> <pem>  Import a mainnet identity from a PEM file"
    echo "  balance [network]    Show wallet or cycles balance for a network (default: local)"
    echo "  help                 Show this help message"
}

command="${1:-status}"

case "$command" in
    status)
        current_id=$(dfx identity whoami)
        principal=$(dfx identity get-principal)
        echo "Active Identity: $current_id"
        echo "Principal ID:    $principal"
        echo ""
        echo "Checking local balance..."
        if dfx ping local &>/dev/null; then
            echo "Local Replica: Running"
            balance=$(dfx ledger balance 2>/dev/null || echo "N/A (Ledger canister not deployed)")
            echo "Local ICP Balance: $balance"
        else
            echo "Local Replica: Not Running"
        fi
        ;;

    use)
        if [ -z "$2" ]; then
            echo "Error: Please specify the identity name to switch to."
            exit 1
        fi
        dfx identity use "$2"
        echo "Switched active identity to: $2"
        ;;

    create)
        if [ -z "$2" ]; then
            echo "Error: Please specify the identity name to create."
            exit 1
        fi
        dfx identity create "$2"
        echo "Created identity: $2"
        ;;

    import)
        if [ -z "$2" ] || [ -z "$3" ]; then
            echo "Error: Usage: $0 import <name> <path-to-pem-file>"
            exit 1
        fi
        dfx identity import "$2" "$3"
        echo "Imported identity '$2' from '$3'"
        ;;

    balance)
        network="${2:-local}"
        echo "Checking balance on network: $network"
        if [ "$network" = "ic" ]; then
            dfx wallet --network ic balance || echo "Could not fetch mainnet cycles wallet balance. Ensure your identity is associated with a cycles wallet."
        else
            if dfx ping local &>/dev/null; then
                dfx wallet balance || echo "Could not fetch local wallet balance."
            else
                echo "Error: Local replica is not running."
                exit 1
            fi
        fi
        ;;

    help|--help|-h)
        print_usage
        ;;

    *)
        echo "Unknown command: $command"
        print_usage
        exit 1
        ;;
esac
