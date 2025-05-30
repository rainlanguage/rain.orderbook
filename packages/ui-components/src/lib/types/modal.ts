import type { SgOrder } from '@rainlanguage/orderbook';
import type { VaultActionArgs, OrderRemoveArgs, DeploymentArgs } from './transaction';

export type VaultActionModalProps = {
	open: boolean;
	args: VaultActionArgs;
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
	rpcUrls: string[],
	orderbook: string,
	inputIOIndex: number,
	outputIOIndex: number,
	pair: string,
	blockNumber?: number
) => void;

export type DebugTradeModalHandler = (hash: string, rpcUrls: string[]) => void;
