#!/bin/bash

# Define all RPC providers in a single file with zone tagging and job label
declare -A rpcs

# Blockops dot + dot-asset
rpcs[wss://polkadot-public-rpc.blockops.network/ws]="polkadot,eu-central,blockops"
rpcs[wss://polkadot-assethub-rpc.blockops.network/ws]="polkadot-assethub,eu-central,blockops"
# Deigenvektor dot
rpcs[wss://polkadot-rpc.deigenvektor.io]="polkadot,eu-central,Deigenvektor"
# Dwellir dot + dot-asset + dot-bridge + dot-col + dot-cor + dot-peo
rpcs[wss://polkadot-rpc.n.dwellir.com]="polkadot,eu-central,Dwellir"
rpcs[wss://asset-hub-polkadot-rpc.n.dwellir.com]="polkadot-assethub,eu-central,Dwellir"
rpcs[wss://bridge-hub-polkadot-rpc.n.dwellir.com]="polkadot-bridgehub,eu-central,Dwellir"
rpcs[wss://collectives-polkadot-rpc.n.dwellir.com]="polkadot-collectives,eu-central,Dwellir"
rpcs[wss://coretime-polkadot-rpc.n.dwellir.com]="polkadot-coretime,eu-central,Dwellir"
rpcs[wss://people-polkadot-rpc.n.dwellir.com]="polkadot-people,eu-central,Dwellir"
# Helixstreet dot
rpcs[wss://rpc-polkadot.helixstreet.io]="polkadot,eu-central,Helixstreet"
# Luckyfriday dot
rpcs[wss://rpc-polkadot.luckyfriday.io]="polkadot,us-east,GlobalStake"
# Onfinality dot
rpcs[wss://polkadot.api.onfinality.io/public-ws]="polkadot,us-east,onfinality"
# Radiumblock dot + dot-asset + dot-bridge
rpcs[wss://polkadot.public.curie.radiumblock.co/ws]="polkadot,us-east,RadiumBlock"
rpcs[wss://statemint.public.curie.radiumblock.co/ws]="polkadot-assethub,us-east,RadiumBlock"
rpcs[wss://bridgehub-polkadot.public.curie.radiumblock.co/ws]="polkadot-bridgehub,us-east,RadiumBlock"
# Stakeworld dot + dot-asset + dot-bridge + dot-col + dot-cor + dot-peo
rpcs[wss://dot-rpc.stakeworld.io]="polkadot,eu-central,stakeworld"
rpcs[wss://dot-rpc.stakeworld.io/assethub]="polkadot-assethub,eu-central,stakeworld"
rpcs[wss://dot-rpc.stakeworld.io/bridgehub]="polkadot-bridgehub,eu-central,stakeworld"
rpcs[wss://dot-rpc.stakeworld.io/collectives]="polkadot-collectives,eu-central,stakeworld"
rpcs[wss://dot-rpc.stakeworld.io/coretime]="polkadot-coretime,eu-central,stakeworld"
rpcs[wss://dot-rpc.stakeworld.io/people]="polkadot-people,eu-central,stakeworld"
# Deigenvektor ksm
rpcs[wss://kusama-rpc.deigenvektor.io]="kusama,eu-central,Deigenvektor"
# Dwellir ksm-bridge + ksm-encointer + ksm-cor + ksm-peo
rpcs[wss://bridge-hub-kusama-rpc.n.dwellir.com]="kusama-bridgehub,eu-central,Dwellir"
rpcs[wss://encointer-kusama-rpc.n.dwellir.com]="kusama-encointer,eu-central,Dwellir"
rpcs[wss://coretime-kusama-rpc.n.dwellir.com]="kusama-coretime,eu-central,Dwellir"
rpcs[wss://people-kusama-rpc.n.dwellir.com]="kusama-people,eu-central,Dwellir"
# Helixstreet ksm
rpcs[wss://rpc-kusama.helixstreet.io]="kusama,eu-central,Helixstreet"
# Luckyfriday ksm
rpcs[wss://rpc-kusama.luckyfriday.io]="kusama,us-east,GlobalStake"
# Onfinality ksm
rpcs[wss://kusama.api.onfinality.io/public-ws]="kusama,us-east,onfinality"
# Radiumblock ksm + ksm-asset
rpcs[wss://kusama.public.curie.radiumblock.co/ws]="kusama,us-east,RadiumBlock"
rpcs[wss://statemine.public.curie.radiumblock.co/ws]="kusama-assethub,us-east,RadiumBlock"
# Stakeworld ksm + ksm-asset + ksm-bridge
rpcs[wss://ksm-rpc.stakeworld.io]="kusama,eu-central,stakeworld"
rpcs[wss://ksm-rpc.stakeworld.io/assethub]="kusama-assethub,eu-central,stakeworld"
rpcs[wss://ksm-rpc.stakeworld.io/bridgehub]="kusama-bridgehub,eu-central,stakeworld"
# Dwellir westend
rpcs[wss://westend-rpc.n.dwellir.com]="westend,eu-central,Dwellir"

