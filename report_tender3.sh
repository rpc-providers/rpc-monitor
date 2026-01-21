#!/bin/bash

# Load the combined config file
config_file="/opt/rpc-monitor/config.sh"
if [[ ! -f "$config_file" ]]; then
    echo "Config file not found: $config_file"
    exit 1
fi
source "$config_file"

PROMETHEUS_SERVER="http://localhost:9090"

# Mapping van network naar chain label in Prometheus
declare -A network_to_chain=(
  [polkadot-coretime]="coretime-polkadot"
  [kusama-coretime]="coretime-kusama"
  [polkadot-bridgehub]="bridge-hub-polkadot"
  [kusama-bridgehub]="bridge-hub-kusama"
  [polkadot-assethub]="asset-hub-polkadot"
  [kusama-assethub]="asset-hub-kusama"
  [polkadot-collectives]="collectives_polkadot"
  [kusama-encointer]="encointer-kusama"
  [polkadot-people]="people-polkadot"
  [kusama-people]="people-kusama"
  [westend]="westend2"
  [polkadot]="polkadot"
  [kusama]="ksmcc3"
)
    start_time=$(date -d "2025-08-01" +%s)
    end_time=$(date -d "2026-01-20" +%s)
    report_date="Tender 3 Report (1 aug 2025 - 20 jan 2026)"

# Functions

fetch_rpc_calls() {
    local chain=$1
    local job=$2
    result=$(curl -s -G "$PROMETHEUS_SERVER/api/v1/query" \
        --data-urlencode "query=sum(increase(substrate_rpc_calls_started{chain=\"$chain\",job=\"$job\"}[173d]))" \
        --data-urlencode "time=$end_time")

    value=$(echo "$result" | jq -r '.data.result[0].value[1]')
    if [ "$value" != "null" ]; then
        printf "%.1f" "$(echo "$value / 1000000" | bc -l)"
    else
        echo "N/A"
    fi
}

# Report Header
echo "## RPC providers report $report_date"
echo ""
echo "- **RPC Calls**: RPC calls in millions"
echo ""

# Table Header
echo "| Endpoint  | Zone         | Network            | RPC Calls (M) |"
echo "|-----------|--------------|--------------------|---------------|"

# Create a temporary list of sorted endpoints based on their associated network
sorted_endpoints=$(for endpoint in "${!rpcs[@]}"; do
    network=$(echo "${rpcs[$endpoint]}" | cut -d, -f1)
    echo "$network|$endpoint"
done | sort | cut -d'|' -f2)

# Data Rows
for endpoint in $sorted_endpoints; do
    entry=${rpcs[$endpoint]}
    network=$(echo "$entry" | cut -d, -f1)
    zone=$(echo "$entry" | cut -d, -f2)
    job=$(echo "$entry" | cut -d, -f3)
    chain=${network_to_chain[$network]}

    if [ -n "$chain" ] && [ -n "$job" ]; then
        rpc_calls=$(fetch_rpc_calls "$chain" "$job")
    else
        rpc_calls="N/A"
    fi

    printf "| %-33s | %-12s | %-20s |  %-22s |
" \
        "$endpoint" "$zone" "$network" "$rpc_calls"
done

