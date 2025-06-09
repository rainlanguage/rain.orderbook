import type { DotrainOrderGui } from '@rainlanguage/orderbook';
import type { TransactionConfirmationProps, TransactionManager } from '@rainlanguage/ui-components';
import { QKEY_ORDERS } from '@rainlanguage/ui-components';
import type { Hex } from 'viem';

export enum AddOrderErrors {
	ADD_ORDER_FAILED = 'Failed to add order',
	MISSING_GUI = 'Order GUI is required',
	MISSING_CONFIG = 'Wagmi config is required',
	NO_ACCOUNT_CONNECTED = 'No wallet address found',
	ERROR_GETTING_ARGS = 'Error getting deployment transaction args',
	ERROR_GETTING_NETWORK_KEY = 'Error getting network key'
}

export type HandleAddOrderDependencies = {
	handleTransactionConfirmationModal: (props: TransactionConfirmationProps) => void;
	errToast: (message: string) => void;
	manager: TransactionManager;
	gui: DotrainOrderGui;
	account: Hex | null;
	subgraphUrl?: string;
};

export const handleAddOrder = async (deps: HandleAddOrderDependencies) => {
	const { gui, account, errToast } = deps;

	const networkKeyResult = gui.getNetworkKey();
	if (networkKeyResult.error) {
		return errToast('Could not deploy: ' + AddOrderErrors.ERROR_GETTING_NETWORK_KEY);
	}
	const network = networkKeyResult.value;

	if (!account) {
		return errToast('Could not deploy: ' + AddOrderErrors.NO_ACCOUNT_CONNECTED);
	}

	const result = await gui.getDeploymentTransactionArgs(account);

	if (result.error) {
		return errToast('Could not deploy: ' + result.error.msg);
	}

	const { approvals, deploymentCalldata, orderbookAddress, chainId } = result.value;

	for (const approval of approvals) {
		const confirmationArgs: TransactionConfirmationProps = {
			open: true,
			modalTitle: 'Approving token spend',
			args: {
				toAddress: approval.token as Hex,
				chainId,
				calldata: approval.calldata as `0x${string}`,
				onConfirm: (hash: Hex) => {
					deps.manager.createApprovalTransaction({
						chainId,
						txHash: hash,
						queryKey: QKEY_ORDERS,
						networkKey: network
					});
				}
			}
		};
		deps.handleTransactionConfirmationModal(confirmationArgs);
	}

	const addOrderArgs: TransactionConfirmationProps = {
		open: true,
		modalTitle: 'Deploying your order',
		args: {
			toAddress: orderbookAddress as Hex,
			chainId,
			calldata: deploymentCalldata as `0x${string}`,
			onConfirm: (hash: Hex) => {
				deps.manager.createAddOrderTransaction({
					chainId,
					txHash: hash,
					queryKey: QKEY_ORDERS,
					networkKey: network,
					subgraphUrl: deps.subgraphUrl || ''
				});
			}
		}
	};
	deps.handleTransactionConfirmationModal(addOrderArgs);
};
