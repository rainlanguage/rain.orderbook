import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api';
import { rpcUrl, orderbookAddress } from './settings';

function useVaultDepositStore() {
  async function call(vaultId: bigint, token: string, amount: bigint) {
    await invoke("vault_deposit", { 
      deposit_args: { 
        vaultId,
        token,
        amount,
      }, 
      transaction_args: { 
        rpcUrl: get(rpcUrl),
        orderbookAddress: get(orderbookAddress),
        derivationIndex: 0,
        chainId: 137,
        maxPriorityFeePerGas: '40000000000',
        maxFeePerGas: '40000000000',
      } 
    });
  }

  return {
    call
  }
}

export const vaultDeposit = useVaultDepositStore();