import {
	type RaindexClient,
	type RaindexOrder,
	type RaindexOrderQuote,
	type TakeOrdersCalldataResult
} from '@rainlanguage/orderbook';
import { type Hex } from 'viem';
import type { TransactionManager, TransactionConfirmationProps } from '@rainlanguage/ui-components';
import type { HandleTakeOrderModal, TakeOrderSubmitParams } from './modal';

export interface TakeOrderHandlerDependencies {
	raindexClient: RaindexClient;
	order: RaindexOrder;
	handleTakeOrderModal: HandleTakeOrderModal;
	handleTransactionConfirmationModal: (props: TransactionConfirmationProps) => void;
	errToast: (message: string) => void;
	manager: TransactionManager;
	account: Hex;
}

export type ExecuteTakeOrderArgs = TakeOrderHandlerDependencies & {
	calldataResult: TakeOrdersCalldataResult;
	inputTokenSymbol: string;
};

function getInputTokenSymbol(order: RaindexOrder, quote: RaindexOrderQuote): string | null {
	const inputIndex = quote.pair.inputIndex;
	const inputs = order.inputsList?.items ?? [];
	if (inputIndex >= inputs.length) {
		return null;
	}
	return inputs[inputIndex].token.symbol ?? 'tokens';
}

async function executeTakeOrder(args: ExecuteTakeOrderArgs): Promise<void> {
	const {
		raindexClient,
		order,
		calldataResult,
		handleTransactionConfirmationModal,
		manager,
		inputTokenSymbol
	} = args;

	handleTransactionConfirmationModal({
		open: true,
		modalTitle: `Taking order for ${inputTokenSymbol}`,
		closeOnConfirm: false,
		args: {
			entity: order,
			toAddress: calldataResult.orderbook as Hex,
			chainId: order.chainId,
			onConfirm: (txHash: Hex) => {
				manager.createTakeOrderTransaction({
					raindexClient,
					txHash,
					chainId: order.chainId,
					queryKey: order.orderHash,
					entity: order
				});
			},
			calldata: calldataResult.calldata as string
		}
	});
}

async function processSubmit(
	deps: TakeOrderHandlerDependencies,
	params: TakeOrderSubmitParams
): Promise<void> {
	const { order, errToast, account } = deps;
	const { quote, mode, amount, priceCap } = params;

	try {
		const inputTokenSymbol = getInputTokenSymbol(order, quote);
		if (!inputTokenSymbol) {
			errToast('Could not determine input token for this order');
			return;
		}

		const calldataResult = await order.getTakeCalldata(
			quote.pair.inputIndex,
			quote.pair.outputIndex,
			account,
			mode,
			amount,
			priceCap
		);
		if (calldataResult.error) {
			errToast(calldataResult.error.readableMsg);
			return;
		}

		await executeTakeOrder({
			...deps,
			calldataResult: calldataResult.value,
			inputTokenSymbol
		});
	} catch {
		errToast('Failed to get calldata for take order.');
	}
}

export async function handleTakeOrder(deps: TakeOrderHandlerDependencies): Promise<void> {
	const { order, handleTakeOrderModal } = deps;

	handleTakeOrderModal({
		open: true,
		order,
		onSubmit: (params: TakeOrderSubmitParams) => {
			processSubmit(deps, params);
		}
	});
}
