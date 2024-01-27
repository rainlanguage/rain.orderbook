import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api';
import { rpcUrl, orderbookAddress, walletDerivationIndex } from '../stores/settings';
import { chainId } from '$lib/stores/chain';

export async function vaultWithdraw(vaultId: bigint, token: string, targetAmount: bigint) {
  await invoke("vault_withdraw", {
    withdrawArgs: {
      vault_id: vaultId.toString(),
      token,
      target_amount: targetAmount.toString(),
    },
    transactionArgs: {
      rpc_url: get(rpcUrl),
      orderbook_address: get(orderbookAddress),
      derivation_index: get(walletDerivationIndex),
      chain_id: get(chainId),
      max_priority_fee_per_gas: '400000000000',
      max_fee_per_gas: '400000000000',
    }
  });
}