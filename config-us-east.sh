#!/bin/bash

# Use an Associative Array for providers definition per zone
declare -A rpcs

rpcs[wss://rpc-kusama.luckyfriday.io]=kusama
rpcs[wss://rpc-polkadot.luckyfriday.io]=polkadot
rpcs[wss://kusama.api.onfinality.io/public-ws]=kusama
rpcs[wss://polkadot.api.onfinality.io/public-ws]=polkadot
rpcs[wss://statemint.api.onfinality.io/public-ws]=polkadot-assethub
rpcs[wss://bridgehub-polkadot.api.onfinality.io/public-ws]=polkadot-bridgehub
