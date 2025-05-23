import type { SgVault } from '@rainlanguage/orderbook';
import type { Hex } from 'viem';
import type { TransactionManager } from '@rainlanguage/ui-components';
import type {
	VaultActionModalProps,
	TransactionConfirmationProps
} from '@rainlanguage/ui-components';
import { getVaultApprovalCalldata, getVaultDepositCalldata } from '@rainlanguage/orderbook';

export interface VaultDepositHandlerDependencies {
	handleDepositModal: (props: VaultActionModalProps) => void;
	handleTransactionConfirmationModal: (props: TransactionConfirmationProps) => void;
	errToast: (message: string) => void;
	manager: TransactionManager;
	network: string;
	orderbookAddress: Hex;
	subgraphUrl: string;
	chainId: number;
	account: Hex;
	rpcUrl: string;
}

async function executeDeposit(
	vault: SgVault,
	amount: bigint,
	deps: VaultDepositHandlerDependencies
) {
	const calldataResult = await getVaultDepositCalldata(vault, amount.toString());
	if (calldataResult.error) {
		return deps.errToast(calldataResult.error.msg);
	} else if (calldataResult.value) {
		deps.handleTransactionConfirmationModal({
			open: true,
			args: {
				entity: vault,
				toAddress: deps.orderbookAddress,
				chainId: deps.chainId,
				onConfirm: (txHash: Hex) => {
					deps.manager.createDepositTransaction({
						subgraphUrl: deps.subgraphUrl,
						txHash,
						chainId: deps.chainId,
						networkKey: deps.network,
						queryKey: vault.id,
						entity: vault,
						amount
					});
				},
				calldata: calldataResult.value
			}
		});
	}
}

export async function handleVaultDeposit(
	vault: SgVault,
	deps: VaultDepositHandlerDependencies
): Promise<void> {
	deps.handleDepositModal({
		open: true,
		args: {
			vault,
			chainId: deps.chainId,
			rpcUrl: deps.rpcUrl,
			subgraphUrl: deps.subgraphUrl,
			account: deps.account
		},
		onSubmit: async (amount: bigint) => {
			const approvalResult = await getVaultApprovalCalldata(deps.rpcUrl, vault, amount.toString());
			if (approvalResult.error) {
				// If getting approval calldata fails, immediately invoke deposit
				await executeDeposit(vault, amount, deps);
			} else if (approvalResult.value) {
				deps.handleTransactionConfirmationModal({
					open: true,
					args: {
						entity: vault,
						toAddress: vault.token.address as Hex,
						chainId: deps.chainId,
						onConfirm: (txHash: Hex) => {
							deps.manager.createApprovalTransaction({
								txHash,
								chainId: deps.chainId,
								networkKey: deps.network,
								queryKey: vault.id,
								entity: vault
							});
							// Immediately invoke deposit after approval
							executeDeposit(vault, amount, deps);
						},
						calldata: approvalResult.value
					}
				});
			}
		}
	});
}
