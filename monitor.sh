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

# auto update
git pull > /dev/null

IFS=$'\n'

pids=$(pgrep monitor | wc -l)
if [ $pids -ne "2" ]; then
  echo "Already running, abort"
  exit
fi

timestamp=$(date +%s%3N)

echo "# HELP rpc_getblockzero time to get block 0" > $prom
echo "# TYPE rpc_getblockzero gauge" >> $prom

for rpc in ${!rpcs[@]}
  do
    network=${rpcs[$rpc]}
    if [ $network = "polkadot" ]; then
      zerohash="0x91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3"
    elif [ $network = "polkadot-assethub" ]; then
      zerohash="0x48239ef607d7928874027a43a67689209727dfb3d3dc5e5b03a39bdc2eda771a"
    elif [ $network = "polkadot-bridgehub" ]; then
      zerohash="0xdcf691b5a3fbe24adc99ddc959c0561b973e329b1aef4c4b22e7bb2ddecb4464"
    elif [ $network = "polkadot-collectives" ]; then
      zerohash="0x46ee89aa2eedd13e988962630ec9fb7565964cf5023bb351f2b6b25c1b68b0b2"
    elif [ $network = "kusama" ]; then
      zerohash="0xb0a8d493285c2df73290dfb7e61f870f17b41801197a149ca93654499ea3dafe"
    elif [ $network = "kusama-assethub" ]; then
      zerohash="0x48239ef607d7928874027a43a67689209727dfb3d3dc5e5b03a39bdc2eda771a"
    elif [ $network = "kusama-bridgehub" ]; then
      zerohash="0x00dcb981df86429de8bbacf9803401f09485366c44efbf53af9ecfab03adc7e5"
    elif [ $network = "kusama-encointer" ]; then
      zerohash="0x7dd99936c1e9e6d1ce7d90eb6f33bea8393b4bf87677d675aa63c9cb3e8c5b5b"
    elif [ $network = "westend" ]; then
      zerohash="0xe143f23803ac50e8f6f8e62695d1ce9e4e1d68aa36c1cd2cfd15340213f3423e"
    fi
    time=$(timeout 120s /usr/bin/time -f "%e" polkadot-js-api --ws "$rpc" rpc.chain.getBlock $zerohash 2>&1 | tail -n1)
    if [ $? -eq 0 ] 
      then 
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

echo "" >> $prom
echo "# HELP rpc_connect time to connect" >> $prom
echo "# TYPE rpc_connect gauge" >> $prom

for rpc in ${!rpcs[@]}
  do
    network=${rpcs[$rpc]}
    rpcdomain=$(echo $rpc | cut -d\/ -f3,4)
    time=$(/usr/bin/timeout --foreground 120s /usr/bin/time -f "%e" /usr/local/bin/websocat -uU $rpc 2>&1)
    if [ $? -eq 0 ]; then
      code=$(curl -LI "https://$rpcdomain" -o /dev/null -w '%{http_code}\n' -s)
      if [ $? -ne 0 ]; then
        code="1"
      fi
      echo "rpc_connect{wss=\"$rpc\",network=\"$network\",zone=\"$zone\"} $time $timestamp" >> $prom
      echo "rpc_error{wss=\"$rpc\",network=\"$network\",zone=\"$zone\",error=\"connect\"} 0 $timestamp" >> $errorprom
      echo "rpc_error{wss=\"$rpc\",network=\"$network\",zone=\"$zone\",error=\"code\"} $code $timestamp" >> $errorprom
    else
      code=$(curl -LI "https://$rpcdomain" -o /dev/null -w '%{http_code}\n' -s)
      if [ $? -ne 0 ]; then
        code="1"
      fi
      echo "rpc_error{wss=\"$rpc\",network=\"$network\",zone=\"$zone\",error=\"connect\"} 1 $timestamp" >> $errorprom
      echo "rpc_error{wss=\"$rpc\",network=\"$network\",zone=\"$zone\",error=\"code\"} $code $timestamp" >> $errorprom
      echo "`date`: $rpc error: code=$code, connect=$time" >> $error
    fi
  done

echo "" >> $prom
echo "# HELP rpc_version rpc version" >> $prom
echo "# TYPE rpc_version gauge" >> $prom

for rpc in ${!rpcs[@]}
  do
    network=${rpcs[$rpc]}
    version=$(timeout 120s polkadot-js-api --ws "$rpc" rpc.system.version 2>&1 | grep version | cut -d\" -f4)
    if [ -z "$version" ]
      then
         echo "`date`: $rpc error: version=$version" >> $error
      else
         echo "rpc_version{wss=\"$rpc\",version=\"$version\",network=\"$network\",zone=\"$zone\"} 1 $timestamp" >> $prom
         echo "rpc_error{wss=\"$rpc\",network=\"$network\",zone=\"$zone\",error=\"version\"} 0 $timestamp" >> $errorprom
    fi
  done

echo "" >> $prom

echo "# HELP rpc_error rpc-error" >> $prom
echo "# TYPE rpc_error gauge" >> $prom
cat $errorprom >> $prom
rm $errorprom
echo "" >> $prom

timestamp2=$(date +%s%3N)
scripttime="$((timestamp2-timestamp))"
scripttimesec=`echo "scale=2;${scripttime}/1000" | bc`
echo "# HELP rpc_script Script run duration" >> $prom
echo "# TYPE rpc_script gauge" >> $prom
echo "rpc_script{zone=\"$zone\"} $scripttimesec $timestamp" >> $prom

echo "" >> $prom

cp $prom $promdest
