import type { SgVault } from '@rainlanguage/orderbook';
import { formatUnits, type Hex } from 'viem';
import type {
	TransactionManager
} from '@rainlanguage/ui-components';
import { getVaultApprovalCalldata, getVaultDepositCalldata } from '@rainlanguage/orderbook';
import { handleVaultActionModal, handleTransactionConfirmationModal } from './modal';

export interface VaultDepositHandlerDependencies {
	vault: SgVault;
	errToast: (message: string) => void;
	manager: TransactionManager;
	network: string;
	orderbookAddress: Hex;
	subgraphUrl: string;
	chainId: number;
	account: Hex;
	rpcUrl: string;
}

export type DepositArgs = VaultDepositHandlerDependencies & { amount: bigint };

async function executeDeposit(args: DepositArgs) {
	const {
		amount,
		vault,
		errToast,
		manager,
		network,
		orderbookAddress,
		subgraphUrl,
		chainId
	} = args;
	const calldataResult = await getVaultDepositCalldata(vault, amount.toString());
	if (calldataResult.error) {
		return errToast(calldataResult.error.msg);
	} else if (calldataResult.value) {
		handleTransactionConfirmationModal({
			open: true,
			modalTitle: `Depositing ${formatUnits(amount, Number(vault.token.decimals))} ${vault.token.symbol}`,
			args: {
				entity: vault,
				toAddress: orderbookAddress,
				chainId: chainId,
				onConfirm: (txHash: Hex) => {
					manager.createDepositTransaction({
						subgraphUrl,
						txHash,
						chainId,
						networkKey: network,
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

export async function handleVaultDeposit(deps: VaultDepositHandlerDependencies): Promise<void> {
	const {
		vault,
		manager,
		network,
		subgraphUrl,
		chainId,
		account,
		rpcUrl
	} = deps;

	handleVaultActionModal({
		open: true,
		actionType: 'deposit',
		args: {
			vault,
			chainId,
			rpcUrl,
			subgraphUrl,
			account
		},
		onSubmit: async (amount: bigint) => {
			const depositArgs = { ...deps, amount };
			const approvalResult = await getVaultApprovalCalldata(rpcUrl, vault, amount.toString());
			console.log("approve result", approvalResult)
			if (approvalResult.error) {
				// If getting approval calldata fails, immediately invoke deposit
				await executeDeposit(depositArgs);
			} else if (approvalResult.value) {
				handleTransactionConfirmationModal({
					open: true,
					modalTitle: `Approving ${vault.token.symbol || 'token'} spend`,
					args: {
						entity: vault,
						toAddress: vault.token.address as Hex,
						chainId: chainId,
						onConfirm: (txHash: Hex) => {
							manager.createApprovalTransaction({
								txHash,
								chainId: chainId,
								networkKey: network,
								queryKey: vault.id,
								entity: vault
							});
							// Immediately invoke deposit after approval
							executeDeposit(depositArgs);
						},
						calldata: approvalResult.value
					}
				});
			}
		}
	});
}
