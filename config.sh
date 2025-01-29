#!/bin/bash

# Define all RPC providers in a single file with zone tagging
declare -A rpcs

# Zone-specific entries for `eu-central`
rpcs[wss://polkadot-public-rpc.blockops.network/ws]="polkadot,eu-central"
rpcs[wss://polkadot-rpc.dwellir.com]="polkadot,eu-central"
rpcs[wss://kusama-rpc.dwellir.com]="kusama,eu-central"
rpcs[wss://asset-hub-polkadot-rpc.dwellir.com]="polkadot-assethub,eu-central"
rpcs[wss://bridge-hub-polkadot-rpc.dwellir.com]="polkadot-bridgehub,eu-central"
rpcs[wss://collectives-polkadot-rpc.dwellir.com]="polkadot-collectives,eu-central"
rpcs[wss://asset-hub-kusama-rpc.dwellir.com]="kusama-assethub,eu-central"
rpcs[wss://bridge-hub-kusama-rpc.dwellir.com]="kusama-bridgehub,eu-central"
rpcs[wss://encointer-kusama-rpc.dwellir.com]="kusama-encointer,eu-central"
rpcs[wss://rpc.dotters.network/polkadot]="polkadot,eu-central"
rpcs[wss://rpc.dotters.network/kusama]="kusama,eu-central"
rpcs[wss://dot-rpc.stakeworld.io]="polkadot,eu-central"
rpcs[wss://dot-rpc.stakeworld.io/assethub]="polkadot-assethub,eu-central"
rpcs[wss://rpc-kusama.helixstreet.io]="kusama,eu-central"
rpcs[wss://rpc-polkadot.helixstreet.io]="polkadot,eu-central"

# Zone-specific entries for `us-east`
rpcs[wss://rpc-kusama.luckyfriday.io]="kusama,us-east"
rpcs[wss://rpc-polkadot.luckyfriday.io]="polkadot,us-east"
rpcs[wss://kusama.api.onfinality.io/public-ws]="kusama,us-east"
rpcs[wss://polkadot.api.onfinality.io/public-ws]="polkadot,us-east"
rpcs[wss://polkadot.public.curie.radiumblock.co/ws]="polkadot,us-east"
rpcs[wss://statemint.public.curie.radiumblock.co/ws]="polkadot-assethub,us-east"
rpcs[wss://bridgehub-polkadot.public.curie.radiumblock.co/ws]="polkadot-bridgehub,us-east"
rpcs[wss://collectives.public.curie.radiumblock.co/ws]="polkadot-collectives,us-east"
rpcs[wss://kusama.public.curie.radiumblock.co/ws]="kusama,us-east"
rpcs[wss://statemine.public.curie.radiumblock.co/ws]="kusama-assethub,us-east"
rpcs[wss://bridgehub-kusama.public.curie.radiumblock.co/ws]="kusama-bridgehub,us-east"

# Zone-specific entries for `ap-southeast`
rpcs[wss://rockx-dot.w3node.com/polka-public-dot/ws]="polkadot,ap-southeast"
rpcs[wss://rockx-ksm.w3node.com/polka-public-ksm/ws]="kusama,ap-southeast"
rpcs[wss://statemint.api.onfinality.io/public-ws]="polkadot-assethub,ap-southeast"
rpcs[wss://bridgehub-polkadot.api.onfinality.io/public-ws]="polkadot-bridgehub,ap-southeast"

