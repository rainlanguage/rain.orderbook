import type { SgOrder } from '@rainlanguage/orderbook';
import type { DepositOrWithdrawArgs, OrderRemoveArgs, DeploymentArgs } from './transaction';
import type { Hex } from 'viem';

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

export type TransactionConfirmationProps = {
	open: boolean;
	args: {
		// Chain ID for switching chains
		chainId: number;
		// Address to send the transaction to
		orderbookAddress: Hex;
		// Function to call when the transaction is confirmed in wallet
		onConfirm: (hash: Hex) => void;
		// Order to generate calldata for
		order: SgOrder;
		// Function to generate calldata for the transaction
		getCalldataFn: () => Promise<string>;
	}
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
