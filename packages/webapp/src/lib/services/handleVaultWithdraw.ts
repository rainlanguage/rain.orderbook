import type { SgVault } from '@rainlanguage/orderbook';
import type { Hex } from 'viem';
import type { TransactionManager } from '@rainlanguage/ui-components';
import type {
	VaultActionModalProps,
	TransactionConfirmationProps
} from '@rainlanguage/ui-components';
import { getVaultWithdrawCalldata } from '@rainlanguage/orderbook';

export interface VaultWithdrawHandlerDependencies {
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

export async function handleVaultWithdraw(
	vault: SgVault,
	deps: VaultWithdrawHandlerDependencies
): Promise<void> {
	deps.handleWithdrawModal({
		open: true,
		args: {
			vault,
			chainId: deps.chainId,
			rpcUrl: deps.rpcUrl,
			subgraphUrl: deps.subgraphUrl,
			account: deps.account
		},
		onSubmit: async (amount: bigint) => {
			let calldata: string;
			try {
				const calldataResult = await getVaultWithdrawCalldata(vault, amount.toString());
				if (calldataResult.error) {
					return deps.errToast(calldataResult.error.msg);
				}
				calldata = calldataResult.value;
				deps.handleTransactionConfirmationModal({
					open: true,
					args: {
						entity: vault,
						orderbookAddress: deps.orderbookAddress,
						chainId: deps.chainId,
						onConfirm: (txHash: Hex) => {
							deps.manager.createWithdrawTransaction({
								subgraphUrl: deps.subgraphUrl,
								txHash,
								chainId: deps.chainId,
								networkKey: deps.network,
								queryKey: vault.id
							});
						},
						calldata
					}
				});
			} catch {
				return deps.errToast('Failed to get calldata for vault withdrawal.');
			}
		}
	});
}
