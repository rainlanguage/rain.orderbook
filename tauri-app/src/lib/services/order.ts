import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api';
import { rpcUrl, orderbookAddress,chainId, subgraphUrl } from '$lib/stores/settings';
import { walletDerivationIndex } from '$lib/stores/wallets';

export async function orderAdd(dotrain: string) {
  await invoke("order_add", {
    addOrderArgs: {
      dotrain,
    },
    transactionArgs: {
      rpc_url: get(rpcUrl),
      orderbook_address: get(orderbookAddress),
      derivation_index: get(walletDerivationIndex),
      chain_id: get(chainId),
    },
  });
}

export async function orderRemove(id: string) {
  await invoke("order_remove", {
    id,
    transactionArgs: {
      rpc_url: get(rpcUrl),
      orderbook_address: get(orderbookAddress),
      derivation_index: get(walletDerivationIndex),
      chain_id: get(chainId),
    },
    subgraphArgs: {
      url: get(subgraphUrl)
    }
  });
}