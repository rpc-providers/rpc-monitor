#!/bin/bash

# Load the combined config file
config_file="/opt/rpc-monitor/config.sh"
if [[ ! -f "$config_file" ]]; then
    echo "Config file not found: $config_file"
    exit 1
fi
source "$config_file"

PROMETHEUS_SERVER="http://localhost:9090"

# Handle optional month input
if [[ -n $1 ]]; then
    month_year=$1
    if [[ ! $month_year =~ ^[0-9]{4}-[0-9]{2}$ ]]; then
        echo "Invalid date format. Use YYYY-MM (e.g., 2024-11)."
        exit 1
    fi
    start_time=$(date -d "${month_year}-01" +%s)
    end_time=$(date -d "${month_year}-01 +1 month -1 second" +%s)
    report_date=$(date -d "${month_year}-01" +"%B %Y")
else
    start_time=$(date -d "30 days ago" +%s)
    end_time=$(date +%s)
    report_date="last 30 days"
fi

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

# Functions
fetch_latency() {
    local metric=$1
    local endpoint=$2
    local network=$3
    local zone=$4
    result=$(curl -s -G "$PROMETHEUS_SERVER/api/v1/query" \
        --data-urlencode "query=avg(avg_over_time(${metric}{wss=\"$endpoint\",zone=\"$zone\",network=\"$network\"}[30d]))" \
        --data-urlencode "time=$end_time")

    if echo "$result" | jq -e '.data.result | length > 0' >/dev/null; then
        value=$(echo "$result" | jq -r '.data.result[0].value[1]')
        printf "%.2f" "$value"
    else
        echo "N/A"
    fi
}

fetch_global_stat() {
    local metric=$1
    local operation=$2

    if [[ "$operation" == "avg" ]]; then
        query="avg(avg_over_time(${metric}[30d]))"
    elif [[ "$operation" == "stddev" ]]; then
        query="avg(stddev_over_time(${metric}[30d]))"
    fi

    result=$(curl -s -G "$PROMETHEUS_SERVER/api/v1/query" \
        --data-urlencode "query=$query" \
        --data-urlencode "time=$end_time")

    if echo "$result" | jq -e '.data.result | length > 0' >/dev/null; then
        value=$(echo "$result" | jq -r '.data.result[0].value[1]')
        printf "%.2f" "$value"
    else
        echo "N/A"
    fi
}

fetch_uptime() {
    local endpoint=$1
    local network=$2
    local zone=$3
    result=$(curl -s -G "$PROMETHEUS_SERVER/api/v1/query" \
        --data-urlencode "query=(1 - (sum_over_time(rpc_error{wss=\"$endpoint\",zone=\"$zone\",network=\"$network\",error=\"blockzero\"}[30d])) / (30 * 24 * 4)) * 100" \
        --data-urlencode "time=$end_time")

    if echo "$result" | jq -e '.data.result | length > 0' >/dev/null; then
        value=$(echo "$result" | jq -r '.data.result[0].value[1]')
        printf "%.2f" "$value"
    else
        echo "N/A"
    fi
}

fetch_version() {
    local endpoint=$1
    local network=$2
    local zone=$3
    result=$(curl -s -G "$PROMETHEUS_SERVER/api/v1/query" \
        --data-urlencode "query=last_over_time(rpc_version{wss=\"$endpoint\",zone=\"$zone\",network=\"$network\"}[20m])" \
        --data-urlencode "time=$end_time")

    if echo "$result" | jq -e '.data.result | length > 0' >/dev/null; then
        version=$(echo "$result" | jq -r '.data.result[-1].metric.version // "N/A"')
        if [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+-[a-f0-9]+$ ]]; then
            echo "${version%%-*}"
        else
            echo "Invalid"
        fi
    else
        echo "N/A"
    fi
}

fetch_rpc_calls() {
    local chain=$1
    local job=$2
    result=$(curl -s -G "$PROMETHEUS_SERVER/api/v1/query" \
        --data-urlencode "query=sum(increase(substrate_rpc_calls_started{chain=\"$chain\",job=\"$job\"}[30d]))" \
        --data-urlencode "time=$end_time")

    value=$(echo "$result" | jq -r '.data.result[0].value[1]')
    if [ "$value" != "null" ]; then
        printf "%.1f" "$(echo "$value / 1000000" | bc -l)"
    else
        echo "N/A"
    fi
}

# Report Header
echo "## RPC providers report for $report_date (`date -d @$start_time` - `date -d @$end_time`)"
echo ""
mean_connect=$(fetch_global_stat "rpc_connect" "avg")
stddev_connect=$(fetch_global_stat "rpc_connect" "stddev")
mean_block=$(fetch_global_stat "rpc_getblockzero" "avg")
stddev_block=$(fetch_global_stat "rpc_getblockzero" "stddev")
echo "- **Connect Time**: Avg = $mean_connect s (Std Dev = $stddev_connect s)"
echo "- **Block Retrieval Time**: Avg = $mean_block s (Std Dev = $stddev_block s)"
echo "- **Uptime**: % of time endpoint could fetch block"
echo "- **Binary Version**: Node version at end of period"
echo "- **RPC Calls**: RPC calls in the last 30 days in millions"
echo ""

# Table Header
echo "| Endpoint                        | Zone         | Network            | Average Connect Time (s) | Average Block Retrieval Time (s) | Uptime (%) | Binary Version         | RPC Calls (M) |"
echo "|---------------------------------|--------------|--------------------|--------------------------|----------------------------------|------------|------------------------|----------------------|"

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

    connect_latency=$(fetch_latency "rpc_connect" "$endpoint" "$network" "$zone")
    block_latency=$(fetch_latency "rpc_getblockzero" "$endpoint" "$network" "$zone")
    uptime=$(fetch_uptime "$endpoint" "$network" "$zone")
    binary_version=$(fetch_version "$endpoint" "$network" "$zone")
    if [ -n "$chain" ] && [ -n "$job" ]; then
        rpc_calls=$(fetch_rpc_calls "$chain" "$job")
    else
        rpc_calls="N/A"
    fi

    printf "| %-33s | %-12s | %-20s | %-24s | %-32s | %-10s | %-22s | %-22s |
" \
        "$endpoint" "$zone" "$network" "$connect_latency" "$block_latency" "$uptime" "$binary_version" "$rpc_calls"
done

