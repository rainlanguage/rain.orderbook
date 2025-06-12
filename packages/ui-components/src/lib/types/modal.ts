import type { SgOrder, SgVault } from '@rainlanguage/orderbook';
import type { VaultActionArgs } from './transaction';
import type { Hex } from 'viem';

export type VaultActionModalProps = {
	open: boolean;
	args: VaultActionArgs;
	onSubmit: (amount: bigint) => void;
};

export type DisclaimerModalProps = {
	open: boolean;
	onAccept: () => void;
};

export type TransactionConfirmationProps = {
	open: boolean;
	// A title for the modal
	modalTitle: string;
	// Close the modal after transaction is confirmed (for approvals that precede deposits)
	closeOnConfirm?: boolean;
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
	rpcUrls: string[],
	orderbook: string,
	inputIOIndex: number,
	outputIOIndex: number,
	pair: string,
	blockNumber?: number
) => void;

export type DebugTradeModalHandler = (hash: string, rpcUrls: string[]) => void;

export type HandleTransactionConfirmationModal = (
	props: TransactionConfirmationProps,
	options?: { timeout?: number }
) => Promise<{ success: boolean; hash?: string }>;
