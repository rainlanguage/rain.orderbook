import type { RaindexClient, RaindexVault } from '@rainlanguage/orderbook';
import { type Hex } from 'viem';
import type { TransactionManager } from '@rainlanguage/ui-components';
import type { TransactionConfirmationProps } from '@rainlanguage/ui-components';

export type WithdrawMultipleModalProps = {
  open: boolean;
  args: {
    vaults: RaindexVault[];
    account: Hex;
  };
  onSubmit: () => void;
};

export interface MultipleVaultsWithdrawHandlerDependencies {
  raindexClient: RaindexClient;
  vaults: RaindexVault[];
  handleWithdrawModal: (props: WithdrawMultipleModalProps) => void;
  handleTransactionConfirmationModal: (props: TransactionConfirmationProps) => void;
  errToast: (message: string) => void;
  manager: TransactionManager;
  account: Hex;
}

export async function handleMultipleVaultsWithdraw(deps: MultipleVaultsWithdrawHandlerDependencies): Promise<void> {
  const {
    raindexClient,
    vaults,
    handleWithdrawModal,
    handleTransactionConfirmationModal,
    errToast,
    manager,
    account
  } = deps;
  // Early return if no vaults are selected
  if (vaults.length === 0) {
    return errToast('No vaults selected for withdrawal.');
  }
  // Early return if vaults are not on the same chain
  if (vaults.every(vault => vault.chainId !== vaults[0].chainId)) {
    return errToast('All vaults must be on the same chain for withdrawal.');
  }

  handleWithdrawModal({
    open: true,
    args: {
      vaults,
      account
    },
    onSubmit: async () => {
      try {
        const calldatas = await Promise.all([
          vaults.map(async (vault) => {
            const calldata = await vault.getWithdrawCalldata(vault.balance.toString());
            if (calldata.error) {
              throw new Error(
                `Failed to get withdrawal calldata for vault ${vault.id}: ${calldata.error?.msg || 'Unknown error'}`
              );
            }
            return calldata.value;
          }),
        ]);

        console.log('calldatas', calldatas);

        // TODO
        // handleTransactionConfirmationModal({
        //   open: true,
        //   modalTitle: `Withdrawing from multiple ${vaults.length} vaults...`,
        //   args: {
        //     toAddress: vaults[0].orderbook, // Assuming all vaults share the same orderbook
        //     chainId: vaults[0].chainId,
        //     onConfirm: (txHash: Hex) => {
        //       manager.createWithdrawTransaction({
        //         raindexClient,
        //         entity: vaults,
        //         txHash,
        //         chainId: vaults[0].chainId,
        //         vaults: vaults.map(vault => vault.id)
        //       });
        //     },
        //     calldata: calldatas.join('') // Concatenate all calldata for the transaction
        //   }
        // });
      } catch {
        return errToast('Failed to get calldata for vault withdrawal.');
      }
    }
  });
}
