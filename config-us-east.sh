#!/bin/bash

# Use an Associative Array for providers definition per zone
declare -A rpcs

rpcs[wss://rpc.ibp.network/polkadot]=polkadot
rpcs[wss://rpc.ibp.network/kusama]=kusama
rpcs[wss://rpc.dotters.network/polkadot]=polkadot
rpcs[wss://rpc.dotters.network/kusama]=kusama
rpcs[wss://rpc-kusama.luckyfriday.io]=kusama
rpcs[wss://rpc-polkadot.luckyfriday.io]=polkadot
rpcs[wss://kusama.api.onfinality.io/public-ws]=kusama
rpcs[wss://polkadot.api.onfinality.io/public-ws]=kusama
