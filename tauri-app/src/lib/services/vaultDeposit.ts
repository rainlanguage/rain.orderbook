import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api';
import { rpcUrl, orderbookAddress, walletDerivationIndex, chainId } from '$lib/stores/settings';

export async function vaultDeposit(vaultId: bigint, token: string, amount: bigint) {
  await invoke("vault_deposit", {
    depositArgs: {
      vault_id: vaultId.toString(),
      token,
      amount: amount.toString(),
    },
    transactionArgs: {
      rpc_url: get(rpcUrl).value,
      orderbook_address: get(orderbookAddress).value,
      derivation_index: get(walletDerivationIndex),
      chain_id: get(chainId),
    }
  });
}