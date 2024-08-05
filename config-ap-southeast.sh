#!/bin/bash

# Use an Associative Array for providers definition
declare -A rpcs

#rpcs[wss://rpc.ibp.network/kusama]=kusama
#rpcs[wss://rpc.ibp.network/polkadot]=polkadot
rpcs[wss://rockx-dot.w3node.com/polka-public-dot/ws]=polkadot
rpcs[wss://rockx-ksm.w3node.com/polka-public-ksm/ws]=kusama
rpcs[wss://rpc.dotters.network/polkadot]=polkadot
rpcs[wss://rpc.dotters.network/kusama]=kusama
