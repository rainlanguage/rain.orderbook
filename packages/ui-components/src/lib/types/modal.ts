import type { SgOrder } from '@rainlanguage/orderbook';
import type { VaultActionArgs, DeploymentArgs } from './transaction';
import type { Hex } from 'viem';

export type VaultActionModalProps = {
	open: boolean;
	args: VaultActionArgs;
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
	// A title for the modal
	modalTitle: string;
	args: {
		// Chain ID for switching chains
		chainId: number;
		// Address to send the transaction to
		orderbookAddress: Hex;
		// Function to call when the transaction is confirmed in wallet
		onConfirm: (hash: Hex) => void;
		// Order to generate calldata for
		order: SgOrder;
		// Calldata for the transaction
		calldata: string;
	};
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
