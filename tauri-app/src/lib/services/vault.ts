import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api';
import { ledgerWalletDerivationIndex } from '$lib/stores/wallets';
import { walletConnectNetwork } from '$lib/stores/walletconnect';
import type { RaindexClient, RaindexVault } from '@rainlanguage/orderbook';

export async function vaultDeposit(
  raindexClient: RaindexClient,
  vault: RaindexVault,
  amount: bigint,
) {
  const chainId = get(walletConnectNetwork);
  const network = raindexClient.getNetworkByChainId(chainId);
  if (network.error) {
    throw new Error(network.error.readableMsg);
  }

  await invoke('vault_deposit', {
    depositArgs: {
      vault_id: vault.vaultId.toString(),
      token: vault.token.address,
      amount: amount.toString(),
    },
    transactionArgs: {
      rpcs: network.value.rpcs,
      orderbook_address: vault.orderbook,
      derivation_index: get(ledgerWalletDerivationIndex),
      chain_id: chainId,
    },
  });
}

export async function vaultWithdraw(
  raindexClient: RaindexClient,
  vault: RaindexVault,
  targetAmount: bigint,
) {
  const chainId = get(walletConnectNetwork);
  const network = raindexClient.getNetworkByChainId(chainId);
  if (network.error) {
    throw new Error(network.error.readableMsg);
  }

  await invoke('vault_withdraw', {
    chainId,
    withdrawArgs: {
      vault_id: vault.vaultId.toString(),
      token: vault.token.address,
      target_amount: targetAmount.toString(),
    },
    transactionArgs: {
      rpcs: network.value.rpcs,
      orderbook_address: vault.orderbook,
      derivation_index: get(ledgerWalletDerivationIndex),
      chain_id: chainId,
    },
  });
}

export async function multiVaultsWithdraw(vaults: RaindexVault[]) {
  const chainId = get(walletConnectNetwork);
  const orderbook = getOrderbookByChainId(chainId);

  await invoke('vaults_withdraw', {
    vaults: vaults.map((vault) => ({
      vault_id: vault.id.toString(),
      token: vault.token,
      target_amount: vault.balance.toString(),
    })),
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
