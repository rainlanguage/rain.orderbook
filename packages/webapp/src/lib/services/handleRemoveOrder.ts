import type { SgOrder } from '@rainlanguage/orderbook';
import type { Hex } from 'viem';
import type { TransactionManager, TransactionConfirmationProps } from '@rainlanguage/ui-components';
import { getRemoveOrderCalldata } from '@rainlanguage/orderbook';

export interface HandleRemoveOrderDependencies {
	handleTransactionConfirmationModal: (props: TransactionConfirmationProps) => void;
	errToast: (message: string) => void;
	manager: TransactionManager;
	network: string;
	orderbookAddress: Hex;
	subgraphUrl: string;
	chainId: number;
	orderHash: string;
}

export async function handleRemoveOrder(
	order: SgOrder,
	deps: HandleRemoveOrderDependencies
): Promise<void> {
	let calldata: string;
	try {
		const calldataResult = await getRemoveOrderCalldata(order);
		if (calldataResult.error) {
			return deps.errToast(calldataResult.error.msg);
		}
		calldata = calldataResult.value;
		deps.handleTransactionConfirmationModal({
			open: true,
			modalTitle: 'Removing order',
			args: {
				entity: order,
				orderbookAddress: deps.orderbookAddress,
				chainId: deps.chainId,
				onConfirm: (txHash: Hex) => {
					deps.manager.createRemoveOrderTransaction({
						subgraphUrl: deps.subgraphUrl,
						txHash,
						queryKey: deps.orderHash,
						chainId: deps.chainId,
						networkKey: deps.network
					});
				},
				calldata
			}
		});
	} catch {
		deps.errToast('Failed to get calldata for order removal.');
	}
}
