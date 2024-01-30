import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api';
import { rpcUrl, orderbookAddress, walletDerivationIndex, subgraphUrl } from '../stores/settings';
import { chainId } from '$lib/stores/chain';
import { MAX_FEE_PER_GAS_PLACEHOLDER } from './gas';

export async function orderRemove(id: string) {
  await invoke("order_remove", {
    id,
    transactionArgs: {
      rpc_url: get(rpcUrl),
      orderbook_address: get(orderbookAddress),
      derivation_index: get(walletDerivationIndex),
      chain_id: get(chainId),
      max_priority_fee_per_gas: MAX_FEE_PER_GAS_PLACEHOLDER,
      max_fee_per_gas: MAX_FEE_PER_GAS_PLACEHOLDER,
    },
    subgraphArgs: {
      url: get(subgraphUrl)
    }
  });
}