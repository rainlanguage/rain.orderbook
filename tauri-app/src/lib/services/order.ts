import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api';
import { rpcUrl, orderbookAddress, walletDerivationIndex, chainId, subgraphUrl } from '$lib/stores/settings';

export async function orderAdd(dotrain: string) {
  await invoke("order_add", {
    addOrderArgs: {
      dotrain,
    },
    transactionArgs: {
      rpc_url: get(rpcUrl).value,
      orderbook_address: get(orderbookAddress).value,
      derivation_index: get(walletDerivationIndex),
      chain_id: get(chainId),
    },
  });
}

export async function orderRemove(id: string) {
  await invoke("order_remove", {
    id,
    transactionArgs: {
      rpc_url: get(rpcUrl).value,
      orderbook_address: get(orderbookAddress).value,
      derivation_index: get(walletDerivationIndex),
      chain_id: get(chainId),
    },
    subgraphArgs: {
      url: get(subgraphUrl).value
    }
  });
}