#!/bin/bash

# Load the combined config file
config_file="/opt/rpc-monitor/config.sh"
if [[ ! -f "$config_file" ]]; then
    echo "Config file not found: $config_file"
    exit 1
fi
source "$config_file"

# Prometheus server address
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
    report_date=$(date -d "${month_year}-01" +"%B %Y")  # Convert to text format
else
    # Default to last 30 days
    start_time=$(date -d "30 days ago" +%s)
    end_time=$(date +%s)
    report_date=$(date -d "30 days ago" +"%B %Y")
fi

# Initialize statistics variables
connection_times=()
block_times=()

# Docusaurus Header
echo "## RPC providers report for $report_date"
echo ""

# Function to fetch latency safely
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

# Function to calculate mean and standard deviation
calculate_stats() {
    local values=($@)
    local sum=0
    local count=${#values[@]}
    local mean=0
    local stddev=0

    # Calculate mean
    for val in "${values[@]}"; do
        sum=$(echo "$sum + $val" | bc -l)
    done
    mean=$(echo "$sum / $count" | bc -l)

    # Calculate standard deviation
    sum=0
    for val in "${values[@]}"; do
        diff=$(echo "$val - $mean" | bc -l)
        sum=$(echo "$sum + ($diff * $diff)" | bc -l)
    done
    stddev=$(echo "sqrt($sum / $count)" | bc -l)

    printf "%.2f %.2f" "$mean" "$stddev"
}

# Iterate through each RPC endpoint in the config
for endpoint in "${!rpcs[@]}"; do
    entry=${rpcs[$endpoint]}
    network=$(echo "$entry" | cut -d, -f1)
    zone=$(echo "$entry" | cut -d, -f2)

    # Fetch metrics
    connect_latency=$(fetch_latency "rpc_connect" "$endpoint" "$network" "$zone")
    block_latency=$(fetch_latency "rpc_getblockzero" "$endpoint" "$network" "$zone")

    # Store valid values for statistics
    if [[ $connect_latency != "N/A" ]]; then
        connection_times+=($connect_latency)
    fi
    if [[ $block_latency != "N/A" ]]; then
        block_times+=($block_latency)
    fi

done

# Calculate and print general statistics
connection_stats=$(calculate_stats "${connection_times[@]}")
block_stats=$(calculate_stats "${block_times[@]}")

mean_connect=$(echo "$connection_stats" | awk '{print $1}')
stddev_connect=$(echo "$connection_stats" | awk '{print $2}')
mean_block=$(echo "$block_stats" | awk '{print $1}')
stddev_block=$(echo "$block_stats" | awk '{print $2}')

echo ""
echo "- **Connect Time**: Monthly average time to connect to the websocket endpoint (Mean = $mean_connect s, Std Dev = $stddev_connect s)."
echo "- **Block Retrieval Time**: Monthly average time to retrieve a block from the rpc server (Mean = $mean_block s, Std Dev = $stddev_block s)."
echo "- **Uptime**: Monthly uptime percentage were the node was reachable without errors."
echo "- **Binary Version**: The binary version running at the end of the month."
echo ""

# Print Markdown Table Header
echo "| Endpoint                        | Zone         | Network            | Average Connect Time (s) | Average Block Retrieval Time (s) | Uptime (%) | Binary Version         |"
echo "|---------------------------------|--------------|--------------------|--------------------------|----------------------------------|------------|------------------------|"

# Function to fetch uptime using a 30-day range
fetch_uptime() {
    local endpoint=$1
    local network=$2
    local zone=$3
    result=$(curl -s -G "$PROMETHEUS_SERVER/api/v1/query" \
        --data-urlencode "query=(1 - (sum(sum_over_time(rpc_error{wss=\"$endpoint\",zone=\"$zone\",network=\"$network\",error=\"blockzero\"}[30d]))) / (30 * 24 * 4)) * 100" \
        --data-urlencode "time=$end_time")

    if echo "$result" | jq -e '.data.result | length > 0' >/dev/null; then
        value=$(echo "$result" | jq -r '.data.result[0].value[1]')
        printf "%.2f" "$value"
    else
        echo "N/A"
    fi
}

# Function to fetch binary version and check format
fetch_version() {
    local endpoint=$1
    local network=$2
    local zone=$3
    result=$(curl -s -G "$PROMETHEUS_SERVER/api/v1/query" \
        --data-urlencode "query=last_over_time(rpc_version{wss=\"$endpoint\",zone=\"$zone\",network=\"$network\"}[1h])" \
        --data-urlencode "time=$end_time")

    if echo "$result" | jq -e '.data.result | length > 0' >/dev/null; then
        version=$(echo "$result" | jq -r '.data.result[-1].metric.version // "N/A"')
        if [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+-[a-f0-9]+$ ]]; then
            echo "${version%%-*}"  # Extract the part before the `-`
        else
            echo "Invalid"
        fi
    else
        echo "N/A"
    fi
}

# Print the detailed endpoint data
for endpoint in "${!rpcs[@]}"; do
    entry=${rpcs[$endpoint]}
    network=$(echo "$entry" | cut -d, -f1)
    zone=$(echo "$entry" | cut -d, -f2)

    # Fetch metrics
    connect_latency=$(fetch_latency "rpc_connect" "$endpoint" "$network" "$zone")
    block_latency=$(fetch_latency "rpc_getblockzero" "$endpoint" "$network" "$zone")
    uptime=$(fetch_uptime "$endpoint" "$network" "$zone")
    binary_version=$(fetch_version "$endpoint" "$network" "$zone")

    # Print results
    printf "| %-31s | %-12s | %-18s | %-19s | %-26s | %-10s | %-22s |\n" \
        "$endpoint" "$zone" "$network" "$connect_latency" "$block_latency" "$uptime" "$binary_version"
done

