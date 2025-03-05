import type { SgOrder } from '@rainlanguage/orderbook/js_api';
import type { DepositOrWithdrawArgs, OrderRemoveArgs, DeploymentArgs } from './transaction';

export type DepositOrWithdrawModalProps = {
	open: boolean;
	args: DepositOrWithdrawArgs;
};

export type OrderRemoveModalProps = {
	open: boolean;
	args: OrderRemoveArgs;
};

export type DeployModalProps = {
	open: boolean;
	args: DeploymentArgs;
};

export type DisclaimerModalProps = {
	open: boolean;
	onAccept: () => void;
};
export type QuoteDebugModalHandler = (
	order: SgOrder,
	rpcUrl: string,
	orderbook: string,
	inputIOIndex: number,
	outputIOIndex: number,
	pair: string,
	blockNumber?: number
) => void;

export type DebugTradeModalHandler = (hash: string, rpcUrl: string) => void;
