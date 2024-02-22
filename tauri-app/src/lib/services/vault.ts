import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api';
import { rpcUrl, orderbookAddress, chainId } from '$lib/stores/settings';
import { walletDerivationIndex } from '$lib/stores/wallets';

export async function vaultDeposit(vaultId: bigint, token: string, amount: bigint) {
  await invoke("vault_deposit", {
    depositArgs: {
      vault_id: vaultId.toString(),
      token,
      amount: amount.toString(),
    },
    transactionArgs: {
      rpc_url: get(rpcUrl),
      orderbook_address: get(orderbookAddress),
      derivation_index: get(walletDerivationIndex),
      chain_id: get(chainId),
    }
  });
}

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
      chain_id: get(chainId)
    }
  });
}