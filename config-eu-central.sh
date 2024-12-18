#!/bin/bash

# Use an Associative Array for providers definition per zone
declare -A rpcs

rpcs[wss://polkadot-public-rpc.blockops.network/ws]=polkadot
rpcs[wss://polkadot-rpc.dwellir.com]=polkadot
rpcs[wss://kusama-rpc.dwellir.com]=kusama
rpcs[wss://asset-hub-polkadot-rpc.dwellir.com]=polkadot-assethub
rpcs[wss://bridge-hub-polkadot-rpc.dwellir.com]=polkadot-bridgehub
rpcs[wss://collectives-polkadot-rpc.dwellir.com]=polkadot-collectives
rpcs[wss://asset-hub-kusama-rpc.dwellir.com]=kusama-assethub
rpcs[wss://bridge-hub-kusama-rpc.dwellir.com]=kusama-bridgehub
rpcs[wss://encointer-kusama-rpc.dwellir.com]=kusama-encointer
rpcs[wss://rpc.dotters.network/polkadot]=polkadot
rpcs[wss://rpc.dotters.network/kusama]=kusama
rpcs[wss://polkadot.public.curie.radiumblock.co/ws]=polkadot
rpcs[wss://statemint.public.curie.radiumblock.co/ws]=polkadot-assethub
rpcs[wss://bridgehub-polkadot.public.curie.radiumblock.co/ws]=polkadot-bridgehub
rpcs[wss://collectives.public.curie.radiumblock.co/ws]=polkadot-collectives
rpcs[wss://kusama.public.curie.radiumblock.co/ws]=kusama
rpcs[wss://statemine.public.curie.radiumblock.co/ws]=kusama-assethub
rpcs[wss://bridgehub-kusama.public.curie.radiumblock.co/ws]=kusama-bridgehub
rpcs[wss://dot-rpc.stakeworld.io]=polkadot
rpcs[wss://dot-rpc.stakeworld.io/assethub]=polkadot-assethub
