#!/bin/bash

# Define all RPC providers in a single file with zone tagging and job label
declare -A rpcs

###############
# DOT endpoints
###############
# Dwellir 
rpcs[wss://polkadot-rpc.n.dwellir.com]="polkadot,eu-central,Dwellir"
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
rpcs[wss://rpc-bulletin.luckyfriday.io]="polkadot-bulletin,us-east,GlobalStake"
# Onfinality 
rpcs[wss://polkadot.api.onfinality.io/public-ws]="polkadot,us-east,onfinality"
rpcs[wss://statemint.api.onfinality.io/public-ws]="polkadot-assethub,us-east,onfinality"
# SimplyStaking
rpcs[wss://spectrum-03.simplystaking.xyz/cG9sa2Fkb3QtMDMtOTFkMmYwZGYtcG9sa2Fkb3Q/LjwBJpV3dIKyWQ/polkadot/mainnet/]="polkadot,us-east,Simply Staking"
rpcs[wss://spectrum-03.simplystaking.xyz/cG9sa2Fkb3QtMDMtOTFkMmYwZGYtcG9sa2Fkb3Q/mgX--uWlEtmNKw/polkadotbridgehub/mainnet/]="polkadot-bridgehub,us-east,Simply Staking"
rpcs[wss://spectrum-03.simplystaking.xyz/cG9sa2Fkb3QtMDMtOTFkMmYwZGYtcG9sa2Fkb3Q/9QbAeudedsupNA/polkadotbulletin/mainnet/]="polkadot-bulletin,us-east,Simply Staking"
# Stakeworld 
rpcs[wss://dot-rpc.stakeworld.io]="polkadot,eu-central,stakeworld"
###############
# KSM endpoints
###############
# Dwellir
rpcs[wss://kusama-rpc.n.dwellir.com]="kusama,eu-central,Dwellir"
rpcs[wss://coretime-kusama-rpc.n.dwellir.com]="kusama-coretime,eu-central,Dwellir"
rpcs[wss://encointer-kusama-rpc.n.dwellir.com]="kusama-encointer,eu-central,Dwellir"
# Helixstreet 
rpcs[wss://rpc-kusama.helixstreet.io]="kusama,eu-central,Helixstreet"
# Luckyfriday
rpcs[wss://rpc-asset-hub-kusama.luckyfriday.io]="kusama-assethub,us-east,GlobalStake"
# Onfinality
rpcs[wss://kusama.api.onfinality.io/public-ws]="kusama,us-east,onfinality"
# SimplyStaking
rpcs[wss://spectrum-03.simplystaking.xyz/cG9sa2Fkb3QtMDMtOTFkMmYwZGYtcG9sa2Fkb3Q/QXq7QZ6Q60NDzA/kusama/mainnet/]="kusama,us-east,Simply Staking"
rpcs[wss://spectrum-03.simplystaking.xyz/cG9sa2Fkb3QtMDMtOTFkMmYwZGYtcG9sa2Fkb3Q/balkpUVauqyv8g/kusamabridgehub/mainnet/]="kusama-bridgehub,us-east,Simply Staking"
# Stakeworld 
rpcs[wss://ksm-rpc.stakeworld.io]="kusama,eu-central,stakeworld"
###############
# WND endpoints
###############
