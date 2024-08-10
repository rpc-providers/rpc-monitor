#!/bin/bash

# Use an Associative Array for providers definition per zone
declare -A rpcs

rpcs[wss://rpc.ibp.network/polkadot]=polkadot
rpcs[wss://rpc.ibp.network/kusama]=kusama
rpcs[wss://polkadot-public-rpc.blockops.network/ws]=polkadot
rpcs[wss://polkadot-rpc.dwellir.com]=polkadot
rpcs[wss://kusama-rpc.dwellir.com]=kusama
rpcs[wss://rpc.dotters.network/polkadot]=polkadot
rpcs[wss://rpc.dotters.network/kusama]=kusama
rpcs[wss://polkadot.public.curie.radiumblock.co/ws]=polkadot
rpcs[wss://kusama.public.curie.radiumblock.co/ws]=kusama
rpcs[wss://dot-rpc.stakeworld.io]=polkadot
rpcs[wss://ksm-rpc.stakeworld.io]=kusama

