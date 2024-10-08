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
    elif [ $network = "kusama" ]; then
      zerohash="0xb0a8d493285c2df73290dfb7e61f870f17b41801197a149ca93654499ea3dafe"
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
