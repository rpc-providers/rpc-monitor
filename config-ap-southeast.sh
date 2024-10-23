#!/bin/bash

# Use an Associative Array for providers definition
declare -A rpcs

rpcs[wss://rockx-dot.w3node.com/polka-public-dot/ws]=polkadot
rpcs[wss://rockx-ksm.w3node.com/polka-public-ksm/ws]=kusama
rpcs[wss://statemint.api.onfinality.io/public-ws]=polkadot-assethub
rpcs[wss://bridgehub-polkadot.api.onfinality.io/public-ws]=polkadot-bridgehub
