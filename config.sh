#!/bin/bash

# Use an Associative Array for providers definition per zone
declare -A rpcs

rpcs[wss://polkadot-public-rpc.blockops.network/ws]=polkadot
rpcs[wss://polkadot-rpc.dwellir.com]=polkadot
#rpcs[wss://polkadot-rpc-tn.dwellir.com]=polkadot
rpcs[wss://rpc.ibp.network/polkadot]=polkadot
rpcs[wss://rpc.dotters.network/polkadot]=polkadot
rpcs[wss://rpc-polkadot.luckyfriday.io]=polkadot
rpcs[wss://polkadot.api.onfinality.io/public-ws]=polkadot
rpcs[wss://polkadot.public.curie.radiumblock.co/ws]=polkadot
rpcs[wss://rockx-dot.w3node.com/polka-public-dot/ws]=polkadot
rpcs[wss://dot-rpc.stakeworld.io]=polkadot
#rpcs[wss://kusama-public-rpc.blockops.network/ws]=kusama
rpcs[wss://kusama-rpc.dwellir.com]=kusama
#rpcs[wss://kusama-rpc-tn.dwellir.com]=kusama
rpcs[wss://rpc.ibp.network/kusama]=kusama
rpcs[wss://rpc.dotters.network/kusama]=kusama
rpcs[wss://rpc-kusama.luckyfriday.io]=kusama
rpcs[wss://kusama.api.onfinality.io/public-ws]=kusama
rpcs[wss://kusama.public.curie.radiumblock.co/ws]=kusama
rpcs[wss://rockx-ksm.w3node.com/polka-public-ksm/ws]=kusama
rpcs[wss://ksm-rpc.stakeworld.io]=kusama
#rpcs[wss://kusama-rpc.publicnode.com]=kusama

