import { getVaultApprovalCalldata, getVaultDepositCalldata, type SgVault } from '@rainlanguage/orderbook';
import type { TransactionManager } from '@rainlanguage/ui-components';
import { getApprovalMessage } from '$lib/types/transaction';
import type { Hex } from 'viem';

export type VaultTransactionData = {
  calldata: string;
  onConfirm: (txHash: Hex) => void;
}

/**
 * Handles the vault deposit process, determining whether approval is needed and returning the appropriate transaction handler
 * @param rpcUrl The RPC URL for the network
 * @param subgraphUrl The subgraph URL
 * @param networkKey The network key
 * @param chainId The chain ID
 * @param vault The vault to deposit into
 * @param amount The amount to deposit
 * @param manager The transaction manager with createApprovalTransaction and createDepositTransaction methods
 * @param errToast Function to display error toast
 * @returns Transaction data or null if process fails
 */
export async function handleVaultDeposit(
  rpcUrl: string,
  subgraphUrl: string,
  networkKey: string,
  chainId: number,
  vault: SgVault, 
  amount: bigint,
  manager: TransactionManager,
  errToast: (message: string) => void
): Promise<VaultTransactionData | null> {
  const approvalResult = await getVaultApprovalCalldata(rpcUrl, vault, amount.toString());
    
  // If approvalResult.value is truthy, the user needs token approval
  if (approvalResult.value) {
    console.log('approvalResult', approvalResult);
    return {
      calldata: approvalResult.value,
      onConfirm: (txHash: Hex) => {
        manager.createApprovalTransaction({
          subgraphUrl,
          txHash,
          chainId,
          networkKey,
          queryKey: vault.id,
          entity: vault
        });
      }
    };
  } 
    
  // If approvalResult.error is truthy, user already has approval, proceed to deposit
  if (approvalResult.error) {
    const depositResult = await getVaultDepositCalldata(vault, amount.toString());
    if (depositResult.error) {
      errToast(depositResult.error.msg);
      return null;
    } 
        
    if (depositResult.value) {
      return {
        calldata: depositResult.value,
        onConfirm: (txHash: Hex) => {
          manager.createDepositTransaction({
            subgraphUrl,
            txHash,
            chainId,
            networkKey,
            queryKey: vault.id,
            entity: vault
          });
        }
      };
    }
  }
    
  return null;
} 