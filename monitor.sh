#!/bin/bash
home="/opt/rpc-monitor"
output="/var/www/prom"
prom="$output/index.txt.new"
errorprom="$output/error.txt.new"
promdest="$output/index.txt"
error="error.log"
source /root/.nvm/nvm.sh
source $home/zone.sh
source $home/config-$zone.sh

cd $home

IFS=$'\n'

pids=$(pgrep monitor | wc -l)
if [ $pids -ne "2" ]; then
  echo "Already running, abort"
  exit
fi

# Error handling
error() {
    echo "Error on line $1"
    echo "Exiting"
    exit 1
}

trap 'error $LINENO' ERR

timestamp=$(date +%s%3N)

echo "# HELP rpc_getblockzero time to get block 0" > $prom
echo "# TYPE rpc_getblockzero gauge" >> $prom

for rpc in ${!rpcs[@]}
  do
    network=${rpcs[$rpc]}
    if [ $network = "polkadot" ]; then
      zerohash="0x91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3"
    elif [ $network = "kusama" ]; then
      zerohash="0xb0a8d493285c2df73290dfb7e61f870f17b41801197a149ca93654499ea3dafe"
    fi
    time=$(timeout 30s /usr/bin/time -f "%e" polkadot-js-api --ws "$rpc" rpc.chain.getBlock $zerohash 2>&1 | tail -n1)
    #timestamp=$(date +%s%3N)
    if [[ $time =~ ^[+-]?[0-9]+([.][0-9]+)?$ ]]; then
      echo "rpc_getblockzero{wss=\"$rpc\",network=\"$network\",zone=\"$zone\"} $time $timestamp" >> $prom
    else
      echo "rpc_error{wss=\"$rpc\",network=\"$network\",zone=\"$zone\"} 1 $timestamp" >> $errorprom
      echo "`date`: $rpc error: blockzero=$time" >> $error
    fi
  done

echo "" >> $prom
echo "# HELP rpc_connect time to connect" >> $prom
echo "# TYPE rpc_connect gauge" >> $prom

for rpc in ${!rpcs[@]}
  do
    network=${rpcs[$rpc]}
    rpcdomain=$(echo $rpc | cut -d\/ -f3,4)
    time=$(timeout 10s /usr/bin/time -f "%e" curl "https://$rpcdomain:443" 2>&1 | tail -n1)
    #timestamp=$(date +%s%3N)
    if [[ $time =~ ^[+-]?[0-9]+([.][0-9]+)?$ ]]; then
      echo "rpc_connect{wss=\"$rpc\",network=\"$network\",zone=\"$zone\"} $time $timestamp" >> $prom
    else
      echo "rpc_error{wss=\"$rpc\",network=\"$network\",zone=\"$zone\"} 1 $timestamp" >> $errorprom
      echo "`date`: $rpc error: connect=$time" >> $error
    fi
  done

echo "" >> $prom
echo "# HELP rpc_version rpc version" >> $prom
echo "# TYPE rpc_version gauge" >> $prom

for rpc in ${!rpcs[@]}
  do
    network=${rpcs[$rpc]}
    version=$(timeout 6s polkadot-js-api --ws "$rpc" rpc.system.version 2>&1 | grep version | cut -d\" -f4)
    #timestamp=$(date +%s%3N)
    if [ -z "$version" ]
      then
         echo "rpc_error{wss=\"$rpc\",network=\"$network\",zone=\"$zone\"} 1 $timestamp" >> $errorprom
         echo "`date`: $rpc error: version=$version" >> $error
      else
         echo "rpc_version{wss=\"$rpc\",version=\"$version\",network=\"$network\",zone=\"$zone\"} 1 $timestamp" >> $prom
    fi
  done

echo "" >> $prom

if [ -f $errorprom ]; then
  echo "# HELP rpc_error rpc-error" >> $prom
  echo "# TYPE rpc_error gauge" >> $prom
  cat $errorprom >> $prom
  rm $errorprom
  echo "" >> $prom
fi

cp $prom $promdest
