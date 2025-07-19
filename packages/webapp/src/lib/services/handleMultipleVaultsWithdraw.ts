import type { RaindexClient, RaindexVault } from '@rainlanguage/orderbook';
import { generateMulticallCalldata } from '@rainlanguage/orderbook';
import { type Hex } from 'viem';
import { QKEY_VAULTS, type TransactionManager } from '@rainlanguage/ui-components';
import type { TransactionConfirmationProps } from '@rainlanguage/ui-components';
import type { TransactionConfirmationModalResult } from './modal';

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
	handleTransactionConfirmationModal: (props: TransactionConfirmationProps) => Promise<TransactionConfirmationModalResult>;
	errToast: (message: string) => void;
	manager: TransactionManager;
	account: Hex;
}

export async function handleMultipleVaultsWithdraw(
	deps: MultipleVaultsWithdrawHandlerDependencies
): Promise<boolean> {
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
		errToast('No vaults selected for withdrawal.');
		return false;
	}
	// Early return if vaults are not on the same chain
	if (vaults.some((vault) => vault.chainId !== vaults[0].chainId)) {
		errToast('All vaults must be on the same chain for withdrawal.');
		return false;
	}

	return new Promise((resolve) => {
		handleWithdrawModal({
			open: true,
			args: {
				vaults,
				account
			},
			onSubmit: async () => {
				try {
					// Validate that all vaults share the same orderbook
					const orderbook = vaults[0].orderbook;
					if (vaults.some((vault) => vault.orderbook !== orderbook)) {
						throw new Error('All vaults must share the same orderbook for batch withdrawal');
					}
					// Get individual withdrawal calldatas
					const calldatas = await Promise.all(
						vaults.map(async (vault) => {
							const result = await vault.getWithdrawCalldata(vault.balance.toString());
							if (result.error) {
								throw new Error(
									`Failed to get withdrawal calldata for vault ${vault.id}: ${result.error.readableMsg || 'Unknown error'}`
								);
							}
							return result.value;
						})
					);
					const calldataResult = await generateMulticallCalldata(calldatas);
					if (calldataResult.error) {
						throw new Error(
							`Failed to generate multicall calldata: ${calldataResult.error.readableMsg || 'Unknown error'}`
						);
					}
					const calldata = calldataResult.value;

					const result = await handleTransactionConfirmationModal({
						open: true,
						modalTitle: `Withdrawing from ${vaults.length} vaults...`,
						args: {
							toAddress: orderbook,
							chainId: vaults[0].chainId,
							calldata,
							onConfirm: async (txHash: Hex) => {
								manager.createMultipleVaultsWithdrawTransaction({
									raindexClient,
									vaults,
									txHash,
									queryKey: QKEY_VAULTS, // Invalidate all vaults
									chainId: vaults[0].chainId,
								});
							}
						}
					});
					resolve(result.success);
				} catch (error) {
					errToast(
						`Failed to generate calldata for vault withdrawal: ${error instanceof Error ? error.message : 'Unknown error'}`
					);
					resolve(false);
				}
			}
		});
	});
}
