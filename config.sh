#!/bin/bash

# Define all RPC providers in a single file with zone tagging
declare -A rpcs

# Blockops dot + dot-asset
rpcs[wss://polkadot-public-rpc.blockops.network/ws]="polkadot,eu-central"
# blockops dot-asset?
# Dwellir dot + dot-asset + dot-bridge + dot-col
rpcs[wss://polkadot-rpc.dwellir.com]="polkadot,eu-central"
rpcs[wss://asset-hub-polkadot-rpc.dwellir.com]="polkadot-assethub,eu-central"
rpcs[wss://bridge-hub-polkadot-rpc.dwellir.com]="polkadot-bridgehub,eu-central"
rpcs[wss://collectives-polkadot-rpc.dwellir.com]="polkadot-collectives,eu-central"
# Helixstreet dot
rpcs[wss://rpc-polkadot.helixstreet.io]="polkadot,eu-central"
# Luckyfriday dot
rpcs[wss://rpc-polkadot.luckyfriday.io]="polkadot,us-east"
# Onfinality dot
rpcs[wss://polkadot.api.onfinality.io/public-ws]="polkadot,us-east"
# Radiumblock dot + dot-asset + dot-bridge
rpcs[wss://polkadot.public.curie.radiumblock.co/ws]="polkadot,us-east"
rpcs[wss://statemint.public.curie.radiumblock.co/ws]="polkadot-assethub,us-east"
rpcs[wss://bridgehub-polkadot.public.curie.radiumblock.co/ws]="polkadot-bridgehub,us-east"
# Stakeworld dot + dot-asset + dot-bridge + dot-col
rpcs[wss://dot-rpc.stakeworld.io]="polkadot,eu-central"
rpcs[wss://dot-rpc.stakeworld.io/assethub]="polkadot-assethub,eu-central"
rpcs[wss://dot-rpc.stakeworld.io/bridgehub]="polkadot-bridgehub,eu-central"
rpcs[wss://dot-rpc.stakeworld.io/collectives]="polkadot-collectives,eu-central"
# Dwellir ksm-bridge + ksm-encointer 
rpcs[wss://bridge-hub-kusama-rpc.dwellir.com]="kusama-bridgehub,eu-central"
rpcs[wss://encointer-kusama-rpc.dwellir.com]="kusama-encointer,eu-central"
# Helixstreet ksm
rpcs[wss://rpc-kusama.helixstreet.io]="kusama,eu-central"
# Luckyfriday ksm
rpcs[wss://rpc-kusama.luckyfriday.io]="kusama,us-east"
# Onfinality ksm
rpcs[wss://kusama.api.onfinality.io/public-ws]="kusama,us-east"
# Radiumblock ksm + ksm-asset
rpcs[wss://kusama.public.curie.radiumblock.co/ws]="kusama,us-east"
rpcs[wss://statemine.public.curie.radiumblock.co/ws]="kusama-assethub,us-east"
# Temporary for debugging
rpcs[wss://bridgehub-kusama.public.curie.radiumblock.co/ws]="kusama-bridgehub,us-east"
rpcs[wss://collectives.public.curie.radiumblock.co/ws]="polkadot-collectives,us-east"
