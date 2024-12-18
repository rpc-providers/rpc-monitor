#!/bin/bash

# Directories and file paths
home="/opt/rpc-monitor"
output="/var/www/prom"
prom="$output/index.txt.new"
errorprom="$output/error.txt.new"
promdest="$output/index.txt"
error="error.log"

# Load environment and configurations
source /root/.nvm/nvm.sh
source $home/zone.sh
source $home/config.sh

cd $home

# Auto update the repository
git pull > /dev/null

# Check if the script is already running
pids=$(pgrep monitor | wc -l)
if [ $pids -ne "2" ]; then
  echo "Already running, abort"
  exit
fi

# Start timestamp
timestamp=$(date +%s%3N)

# Prometheus metrics headers
echo "# HELP rpc_getblockzero time to get block 0" > $prom
echo "# TYPE rpc_getblockzero gauge" >> $prom

# Filter RPCs by zone
declare -A filtered_rpcs
for rpc in "${!rpcs[@]}"; do
  entry=${rpcs[$rpc]}
  network=$(echo "$entry" | cut -d, -f1)
  rpc_zone=$(echo "$entry" | cut -d, -f2)
  if [ "$rpc_zone" == "$zone" ]; then
    filtered_rpcs[$rpc]=$network
  fi
done

# Monitor RPC endpoints for block zero retrieval time
for rpc in "${!filtered_rpcs[@]}"; do
  network=${filtered_rpcs[$rpc]}
  case $network in
    "polkadot") zerohash="0x91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3" ;;
    "kusama") zerohash="0xb0a8d493285c2df73290dfb7e61f870f17b41801197a149ca93654499ea3dafe" ;;
    "polkadot-assethub") zerohash="0x48239ef607d7928874027a43a67689209727dfb3d3dc5e5b03a39bdc2eda771a" ;;
    "kusama-assethub") zerohash="0x48239ef607d7928874027a43a67689209727dfb3d3dc5e5b03a39bdc2eda771a" ;;
    "polkadot-bridgehub") zerohash="0xdcf691b5a3fbe24adc99ddc959c0561b973e329b1aef4c4b22e7bb2ddecb4464" ;;
    "kusama-bridgehub") zerohash="0x00dcb981df86429de8bbacf9803401f09485366c44efbf53af9ecfab03adc7e5" ;;
    "kusama-encointer") zerohash="0x7dd99936c1e9e6d1ce7d90eb6f33bea8393b4bf87677d675aa63c9cb3e8c5b5b" ;;
    "westend") zerohash="0xe143f23803ac50e8f6f8e62695d1ce9e4e1d68aa36c1cd2cfd15340213f3423e" ;;
    *) zerohash="" ;;
  esac
  time=$(timeout 120s /usr/bin/time -f "%e" polkadot-js-api --ws "$rpc" rpc.chain.getBlock $zerohash 2>&1 | tail -n1)
  if [ $? -eq 0 ]; then
    if [[ $time =~ ^[+-]?[0-9]+([.][0-9]+)?$ ]]; then
      echo "rpc_getblockzero{wss=\"$rpc\",network=\"$network\",zone=\"$zone\"} $time $timestamp" >> $prom
      echo "rpc_error{wss=\"$rpc\",network=\"$network\",zone=\"$zone\",error=\"blockzero\"} 0 $timestamp" >> $errorprom
    else
      echo "rpc_error{wss=\"$rpc\",network=\"$network\",zone=\"$zone\",error=\"blockzero\"} 1 $timestamp" >> $errorprom
      echo "`date`: $rpc error: blockzero=$time" >> $error
    fi
  else
    echo "rpc_error{wss=\"$rpc\",network=\"$network\",zone=\"$zone\",error=\"blockzero\"} 1 $timestamp" >> $errorprom
    echo "`date`: $rpc error: blockzero=$time" >> $error
  fi
done

# Metrics: Connection time
echo "" >> $prom  # Add newline after section
echo "# HELP rpc_connect time to connect" >> $prom
echo "# TYPE rpc_connect gauge" >> $prom

for rpc in "${!filtered_rpcs[@]}"; do
  network=${filtered_rpcs[$rpc]}
  rpcdomain=$(echo $rpc | cut -d\/ -f3,4)
  time=$(/usr/bin/timeout --foreground 120s /usr/bin/time -f "%e" /usr/local/bin/websocat -uU $rpc 2>&1)
  if [ $? -eq 0 ]; then
    code=$(curl -LI "https://$rpcdomain" -o /dev/null -w '%{http_code}\n' -s)
    if [ $? -ne 0 ]; then code="1"; fi
    echo "rpc_connect{wss=\"$rpc\",network=\"$network\",zone=\"$zone\"} $time $timestamp" >> $prom
    echo "rpc_error{wss=\"$rpc\",network=\"$network\",zone=\"$zone\",error=\"connect\"} 0 $timestamp" >> $errorprom
    echo "rpc_error{wss=\"$rpc\",network=\"$network\",zone=\"$zone\",error=\"code\"} $code $timestamp" >> $errorprom
  else
    code=$(curl -LI "https://$rpcdomain" -o /dev/null -w '%{http_code}\n' -s)
    if [ $? -ne 0 ]; then code="1"; fi
    echo "rpc_error{wss=\"$rpc\",network=\"$network\",zone=\"$zone\",error=\"connect\"} 1 $timestamp" >> $errorprom
    echo "rpc_error{wss=\"$rpc\",network=\"$network\",zone=\"$zone\",error=\"code\"} $code $timestamp" >> $errorprom
    echo "`date`: $rpc error: code=$code, connect=$time" >> $error
  fi
done

# Metrics: Binary version
echo "" >> $prom  # Add newline after section
echo "# HELP rpc_version binary version" >> $prom
echo "# TYPE rpc_version gauge" >> $prom

for rpc in "${!filtered_rpcs[@]}"; do
  network=${filtered_rpcs[$rpc]}
  version=$(timeout 120s polkadot-js-api --ws "$rpc" rpc.system.version 2>&1 | grep version | cut -d\" -f4)
  if [ -z "$version" ]; then
    echo "rpc_error{wss=\"$rpc\",network=\"$network\",zone=\"$zone\",error=\"version\"} 1 $timestamp" >> $errorprom
    echo "`date`: $rpc error: version=$version" >> $error
  else
    echo "rpc_version{wss=\"$rpc\",network=\"$network\",zone=\"$zone\",version=\"$version\"} 1 $timestamp" >> $prom
    echo "rpc_error{wss=\"$rpc\",network=\"$network\",zone=\"$zone\",error=\"version\"} 0 $timestamp" >> $errorprom
  fi
done

# Finalize Prometheus metrics
echo "" >> $prom  # Add newline after section
echo "# HELP rpc_error rpc-error" >> $prom
echo "# TYPE rpc_error gauge" >> $prom
cat $errorprom >> $prom
rm $errorprom

# Script duration metrics
echo "" >> $prom  # Add newline after section
timestamp2=$(date +%s%3N)
scripttime="$((timestamp2-timestamp))"
scripttimesec=$(bc <<< "scale=2; $scripttime / 1000")
echo "# HELP rpc_script Script run duration" >> $prom
echo "# TYPE rpc_script gauge" >> $prom
echo "rpc_script{zone=\"$zone\"} $scripttimesec $timestamp" >> $prom

# Replace the old Prometheus file
cp $prom $promdest

