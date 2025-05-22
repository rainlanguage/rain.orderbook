import type { DeploymentTransactionArgs } from '@rainlanguage/orderbook';
import type { TransactionConfirmationProps, TransactionManager } from '@rainlanguage/ui-components';
import { QKEY_ORDERS } from '@rainlanguage/ui-components';
import type { Hex } from 'viem';


export type HandleAddOrderDependencies = {
	args: DeploymentTransactionArgs;
	handleTransactionConfirmationModal: (props: TransactionConfirmationProps) => void;
	errToast: (message: string) => void;
	manager: TransactionManager;
	network: string;
	orderbookAddress: Hex;
	subgraphUrl: string;
	chainId: number;
};

export const handleAddOrder = async (deps: HandleAddOrderDependencies) => {
	const { approvals, deploymentCalldata, chainId, orderbookAddress } = deps.args;
	for (const approval of approvals) {
		const confirmationArgs: TransactionConfirmationProps = {
			open: true,
			args: {
				toAddress: approval.token as Hex,
				chainId,
				calldata: approval.calldata as `0x${string}`,
				onConfirm: (hash: Hex) => {
					deps.manager.createApprovalTransaction({
						...deps.args,
						txHash: hash,
						queryKey: QKEY_ORDERS,
						networkKey: deps.network
					});
				}
			}
		};
		await deps.handleTransactionConfirmationModal(confirmationArgs);
	}

	const addOrderArgs: TransactionConfirmationProps = {
		open: true,
		args: {
			toAddress: orderbookAddress as Hex,
			chainId,
			calldata: deploymentCalldata as `0x${string}`,
			onConfirm: (hash: Hex) => {
				deps.manager.createAddOrderTransaction({
					...deps.args,
					txHash: hash,
					queryKey: QKEY_ORDERS,
					networkKey: deps.network,
					subgraphUrl: deps.subgraphUrl
				});
			}
		}
	};
	await deps.handleTransactionConfirmationModal(addOrderArgs);
};
