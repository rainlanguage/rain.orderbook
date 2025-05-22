import type { SgOrder, SgVault } from '@rainlanguage/orderbook';
import type { DeploymentArgs, VaultActionArgs } from './transaction';
import type { Hex } from 'viem';

export type VaultActionModalProps = {
	open: boolean;
	args: VaultActionArgs;
	onSubmit: (amount: bigint) => void;
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
		// Chain ID for switching chains.
		chainId: number;
		// Address to send the transaction to.
		toAddress: Hex;
		// Function to call when the transaction is confirmed in wallet.
		onConfirm: (hash: Hex) => void;
		// Entity to generate calldata for (optional, not used for approval transactions when adding orders).
		entity?: SgOrder | SgVault;
		// Calldata for the transaction.
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
