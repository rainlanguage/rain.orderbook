import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api';
import { rpcUrl, orderbookAddress, chainId } from '$lib/stores/settings';
import { walletDerivationIndex } from '$lib/stores/wallets';

export async function vaultDeposit(vaultId: bigint, token: string, amount: bigint) {
  const [rpc_url, orderbook_address, chain_id] = await Promise.all([
    rpcUrl.load(),
    orderbookAddress.load(),
    chainId.load()
  ]);

  await invoke("vault_deposit", {
    depositArgs: {
      vault_id: vaultId.toString(),
      token,
      amount: amount.toString(),
    },
    transactionArgs: {
      rpc_url,
      orderbook_address,
      derivation_index: get(walletDerivationIndex),
      chain_id
    }
  });
}

export async function vaultWithdraw(vaultId: bigint, token: string, targetAmount: bigint) {
  const [rpc_url, orderbook_address, chain_id] = await Promise.all([
    rpcUrl.load(),
    orderbookAddress.load(),
    chainId.load()
  ]);

  await invoke("vault_withdraw", {
    withdrawArgs: {
      vault_id: vaultId.toString(),
      token,
      target_amount: targetAmount.toString(),
    },
    transactionArgs: {
      rpc_url,
      orderbook_address,
      derivation_index: get(walletDerivationIndex),
      chain_id,
    }
  });
}