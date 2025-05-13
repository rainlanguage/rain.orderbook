import type { Hex } from 'viem';
import { sendTransaction, switchChain, waitForTransactionReceipt } from '@wagmi/core';
import { getExplorerLink } from '../services/getExplorerLink';
import { TransactionStatusMessage, TransactionErrorMessage } from '$lib/types/transaction';
import type { RemoveOrderTransactionArgs } from '$lib/types/transaction';
import {
	awaitSubgraphIndexing,
	getRemoveOrderConfig
} from '$lib/services/awaitTransactionIndexing';
import type { Config } from '@wagmi/core';

export type RemoveOrderTransaction = {
	message: TransactionStatusMessage;
	state:
		| RemoveOrderTransactionState
		| ConfirmedRemoveOrderTransactionState
		| FailedRemoveOrderTransactionState;
};

type RemoveOrderTransactionState = {
	status: TransactionStatusMessage;
	network: string;
	message: string;
};

type ConfirmedTransactionData = {
	hash: Hex;
	explorerLink: string;
};

type FailedRemoveOrderTransactionState = RemoveOrderTransactionState & {
	errorDetails: TransactionErrorMessage;
};
type ConfirmedRemoveOrderTransactionState = RemoveOrderTransactionState & ConfirmedTransactionData;

export class RemoveOrder implements RemoveOrderTransaction {
	private config: Config;
	private orderbookAddress: Hex;
	private removeOrderCalldata: Hex;
	private chainId: number;
	private subgraphUrl: string;

	public state:
		| RemoveOrderTransactionState
		| ConfirmedRemoveOrderTransactionState
		| FailedRemoveOrderTransactionState;

	constructor(args: RemoveOrderTransactionArgs) {
		this.config = args.config;
		this.orderbookAddress = args.orderbookAddress;
		this.removeOrderCalldata = args.removeOrderCalldata as Hex;
		this.chainId = args.chainId;
		this.subgraphUrl = args.subgraphUrl;
		this.state = {
			status: TransactionStatusMessage.IDLE,
			network: '',
			message: TransactionStatusMessage.IDLE
		};
	}

	public get message(): TransactionStatusMessage {
		return this.state.status;
	}

	private updateState(
		newState: Partial<
			| RemoveOrderTransactionState
			| ConfirmedRemoveOrderTransactionState
			| FailedRemoveOrderTransactionState
		>
	) {
		this.state = { ...this.state, ...newState };
	}

	public async execute(): Promise<void> {
		this.updateState({ status: TransactionStatusMessage.IDLE, message: 'Starting order removal.' });
		if (!(await this.switchNetwork())) return;
		const txHash = await this.sendRemoveOrderTransaction();
		if (!txHash) return;
		if (!(await this.waitForTxReceipt(txHash))) return;
		await this.indexOrderRemoval(txHash);
	}

	private async switchNetwork(): Promise<boolean> {
		try {
			this.updateState({ message: 'Switching network...' });
			await switchChain(this.config, { chainId: this.chainId });
			this.updateState({ message: 'Network switched successfully.' });
			return true;
		} catch (error) {
			this.updateState({
				status: TransactionStatusMessage.ERROR,
				errorDetails: TransactionErrorMessage.SWITCH_CHAIN_FAILED,
				message: 'Failed to switch network.'
			});
			return false;
		}
	}

	private async sendRemoveOrderTransaction(): Promise<Hex | null> {
		try {
			this.updateState({
				status: TransactionStatusMessage.PENDING_WALLET,
				message: 'Please confirm order removal in your wallet...'
			});
			const hash = await sendTransaction(this.config, {
				to: this.orderbookAddress,
				data: this.removeOrderCalldata
			});
			const explorerLink = await getExplorerLink(hash, this.chainId, 'tx');
			this.updateState({
				hash,
				status: TransactionStatusMessage.PENDING_REMOVE_ORDER,
				message: 'Processing order removal...',
				explorerLink
			});
			return hash;
		} catch (error) {
			this.updateState({
				status: TransactionStatusMessage.ERROR,
				errorDetails: TransactionErrorMessage.USER_REJECTED_TRANSACTION,
				message: 'Failed to send transaction.'
			});
			return null;
		}
	}

	private async waitForTxReceipt(hash: Hex): Promise<boolean> {
		try {
			this.updateState({ message: `Waiting for transaction receipt (hash: ${hash})...` });
			await waitForTransactionReceipt(this.config, { hash });
			this.updateState({ message: 'Transaction receipt received.' });
			return true;
		} catch (error) {
			this.updateState({
				status: TransactionStatusMessage.ERROR,
				errorDetails: TransactionErrorMessage.REMOVE_ORDER_FAILED,
				message: 'Failed to get transaction receipt.'
			});
			return false;
		}
	}

	private async indexOrderRemoval(txHash: Hex): Promise<void> {
		this.updateState({
			status: TransactionStatusMessage.PENDING_SUBGRAPH,
			message: 'Waiting for order removal to be indexed...'
		});
		try {
			const result = await awaitSubgraphIndexing(
				getRemoveOrderConfig(this.subgraphUrl, txHash, 'Order removed successfully')
			);

			if (result.error) {
				this.updateState({
					status: TransactionStatusMessage.ERROR,
					errorDetails: TransactionErrorMessage.TIMEOUT,
					message: 'Subgraph indexing timed out.'
				});
			} else if (result.value) {
				this.updateState({
					status: TransactionStatusMessage.SUCCESS,
					hash: result.value.txHash as Hex,
					message: result.value.successMessage || 'Order removal indexed successfully.'
				});
			}
		} catch (error) {
			this.updateState({
				status: TransactionStatusMessage.ERROR,
				errorDetails: TransactionErrorMessage.TIMEOUT,
				message: 'Failed to index order removal.'
			});
		}
	}
}
