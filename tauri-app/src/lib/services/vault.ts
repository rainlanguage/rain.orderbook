import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api';
import { ledgerWalletDerivationIndex } from '$lib/stores/wallets';
import { getOrderbookByChainId } from '$lib/utils/getOrderbookByChainId';
import { walletConnectNetwork } from '$lib/stores/walletconnect';

export async function vaultDeposit(vaultId: bigint, token: string, amount: bigint) {
  const chainId = get(walletConnectNetwork);
  const orderbook = getOrderbookByChainId(chainId);

  await invoke('vault_deposit', {
    depositArgs: {
      vault_id: vaultId.toString(),
      token,
      amount: amount.toString(),
    },
    transactionArgs: {
      rpcs: orderbook.network.rpcs,
      orderbook_address: orderbook.address,
      derivation_index: get(ledgerWalletDerivationIndex),
      chain_id: chainId,
    },
  });
}

export async function vaultWithdraw(vaultId: bigint, token: string, targetAmount: bigint) {
  const chainId = get(walletConnectNetwork);
  const orderbook = getOrderbookByChainId(chainId);

  await invoke('vault_withdraw', {
    chainId,
    withdrawArgs: {
      vault_id: vaultId.toString(),
      token,
      target_amount: targetAmount.toString(),
    },
    transactionArgs: {
      rpcs: orderbook.network.rpcs,
      orderbook_address: orderbook.address,
      derivation_index: get(ledgerWalletDerivationIndex),
      chain_id: chainId,
    },
  });
}

export async function vaultDepositCalldata(vaultId: bigint, token: string, amount: bigint) {
  return await invoke('vault_deposit_calldata', {
    depositArgs: {
      vault_id: vaultId.toString(),
      token,
      amount: amount.toString(),
    },
  });
}

export async function vaultDepositApproveCalldata(vaultId: bigint, token: string, amount: bigint) {
  const chainId = get(walletConnectNetwork);
  const orderbook = getOrderbookByChainId(get(walletConnectNetwork));

  return invoke('vault_deposit_approve_calldata', {
    depositArgs: {
      vault_id: vaultId.toString(),
      token,
      amount: amount.toString(),
    },
    transactionArgs: {
      rpcs: orderbook.network.rpcs,
      orderbook_address: orderbook.address,
      derivation_index: get(ledgerWalletDerivationIndex),
      chain_id: chainId,
    },
  });
}
