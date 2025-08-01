#!/bin/bash

# Define all RPC providers in a single file with zone tagging and job label
declare -A rpcs

# Blockops
rpcs[wss://polkadot-public-rpc.blockops.network/ws]="polkadot,eu-central,blockops"
# Dwellir 
rpcs[wss://asset-hub-polkadot-rpc.n.dwellir.com]="polkadot-assethub,eu-central,Dwellir"
rpcs[wss://bridge-hub-polkadot-rpc.n.dwellir.com]="polkadot-bridgehub,eu-central,Dwellir"
rpcs[wss://collectives-polkadot-rpc.n.dwellir.com]="polkadot-collectives,eu-central,Dwellir"
rpcs[wss://coretime-polkadot-rpc.n.dwellir.com]="polkadot-coretime,eu-central,Dwellir"
rpcs[wss://people-polkadot-rpc.n.dwellir.com]="polkadot-people,eu-central,Dwellir"
# Helixstreet
rpcs[wss://rpc-polkadot.helixstreet.io]="polkadot,eu-central,Helixstreet"
# Luckyfriday
rpcs[wss://rpc-polkadot.luckyfriday.io]="polkadot,us-east,GlobalStake"
rpcs[wss://rpc-asset-hub-polkadot.luckyfriday.io]="polkadot-assethub,us-east,GlobalStake"
rpcs[wss://rpc-bridge-hub-polkadot.luckyfriday.io]="polkadot-bridgehub,us-east,GlobalStake"
# Onfinality 
rpcs[wss://polkadot.api.onfinality.io/public-ws]="polkadot,us-east,onfinality"
rpcs[wss://statemint.api.onfinality.io/public-ws]="polkadot-assethub,us-east,onfinality"
rpcs[wss://bridgehub-polkadot.api.onfinality.io/public-ws]="polkadot-bridgehub,us-east,onfinality"
rpcs[wss://collectives-polkadot.api.onfinality.io/public-ws]="polkadot-collectives,us-east,onfinality"
rpcs[wss://coretime-polkadot.api.onfinality.io/public-ws]="polkadot-coretime,us-east,onfinality"
# Radiumblock
rpcs[wss://polkadot.public.curie.radiumblock.co/ws]="polkadot,us-east,RadiumBlock"
# Simplystaking
rpcs[wss://spectrum-01.simplystaking.xyz/cG9sa2Fkb3QtMDEtOTFkMmYwZGYtcG9sa2Fkb3Q/uj77qtNEPueW1w/polkadot/mainnet/]="polkadot,eu-central,simplystaking"
# Stakeworld 
rpcs[wss://dot-rpc.stakeworld.io]="polkadot,eu-central,stakeworld"
rpcs[wss://dot-rpc.stakeworld.io/assethub]="polkadot-assethub,eu-central,stakeworld"
###############
# KSM endpoints
###############
# Dwellir
rpcs[wss://kusama-rpc.n.dwellir.com]="kusama,eu-central,Dwellir"
rpcs[wss://coretime-kusama-rpc.n.dwellir.com]="kusama-coretime,eu-central,Dwellir"
rpcs[wss://encointer-kusama-rpc.n.dwellir.com]="kusama-encointer,eu-central,Dwellir"
rpcs[wss://people-kusama-rpc.n.dwellir.com]="kusama-people,eu-central,Dwellir"
# Helixstreet 
rpcs[wss://rpc-kusama.helixstreet.io]="kusama,eu-central,Helixstreet"
# Luckyfriday
rpcs[wss://rpc-kusama.luckyfriday.io]="kusama,us-east,GlobalStake"
# Onfinality
rpcs[wss://assethub-kusama.api.onfinality.io/public-ws]="kusama-assethub,us-east,onfinality"
rpcs[wss://bridgehubi-kusama.api.onfinality.io/public-ws]="kusama-bridgehub,us-east,onfinality"
# Radiumblock
rpcs[wss://statemine.public.curie.radiumblock.co/ws]="kusama-assethub,us-east,RadiumBlock"
rpcs[wss://bridgehub-kusama.public.curie.radiumblock.co/ws]="kusama-bridgehub,us-east,RadiumBlock"
# Stakeworld 
rpcs[wss://ksm-rpc.stakeworld.io]="kusama,eu-central,stakeworld"
# Dwellir westend
rpcs[wss://westend-rpc.n.dwellir.com]="westend,eu-central,Dwellir"

