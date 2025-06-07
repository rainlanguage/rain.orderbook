import type { SgVault } from '@rainlanguage/orderbook';
import { formatUnits, type Hex } from 'viem';
import type { TransactionManager } from '@rainlanguage/ui-components';
import type {
	VaultActionModalProps,
	TransactionConfirmationProps
} from '@rainlanguage/ui-components';
import { getVaultWithdrawCalldata } from '@rainlanguage/orderbook';

export interface VaultWithdrawHandlerDependencies {
	vault: SgVault;
	handleWithdrawModal: (props: VaultActionModalProps) => void;
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

export async function handleVaultWithdraw(deps: VaultWithdrawHandlerDependencies): Promise<void> {
	const {
		vault,
		handleWithdrawModal,
		handleTransactionConfirmationModal,
		errToast,
		manager,
		network,
		orderbookAddress,
		subgraphUrl,
		chainId,
		account,
		rpcUrl
	} = deps;
	handleWithdrawModal({
		open: true,
		args: {
			vault,
			chainId,
			rpcUrl,
			subgraphUrl,
			account
		},
		onSubmit: async (amount: bigint) => {
			let calldata: string;
			try {
				const calldataResult = await getVaultWithdrawCalldata(vault, amount.toString());
				if (calldataResult.error) {
					return errToast(calldataResult.error.msg);
				}
				calldata = calldataResult.value;
				handleTransactionConfirmationModal({
					open: true,
					modalTitle: `Withdrawing ${formatUnits(amount, Number(vault.token.decimals))} ${vault.token.symbol}...`,
					args: {
						entity: vault,
						orderbookAddress,
						chainId,
						onConfirm: (txHash: Hex) => {
							manager.createWithdrawTransaction({
								subgraphUrl,
								txHash,
								chainId,
								networkKey: network,
								queryKey: vault.id
							});
						},
						calldata
					}
				});
			} catch {
				return errToast('Failed to get calldata for vault withdrawal.');
			}
		}
	});
}
