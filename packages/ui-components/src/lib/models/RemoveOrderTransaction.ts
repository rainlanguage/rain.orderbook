import type { Hex } from 'viem';
import { waitForTransactionReceipt } from '@wagmi/core';
import { TransactionStatusMessage, TransactionErrorMessage } from '$lib/types/transaction';
import type { RemoveOrderTransactionArgs } from '$lib/types/transaction';
import {
	awaitSubgraphIndexing,
	getRemoveOrderConfig
} from '$lib/services/awaitTransactionIndexing';
import type { Config } from '@wagmi/core';
import { getExplorerLink } from '$lib/services/getExplorerLink';
import { match, P } from 'ts-pattern';

type RemoveOrderTransactionState = 
    | { status: TransactionStatusMessage.IDLE; message: string; explorerLink: string }
    | { status: TransactionStatusMessage.PENDING_REMOVE_ORDER; message: string; explorerLink: string }
    | { status: TransactionStatusMessage.PENDING_SUBGRAPH; message: string; explorerLink: string }
    | { status: TransactionStatusMessage.SUCCESS; message: string; explorerLink: string; hash?: Hex }
    | { status: TransactionStatusMessage.ERROR; message: string; explorerLink: string; errorDetails?: TransactionErrorMessage };

export type RemoveOrderTransaction = {
	message: TransactionStatusMessage;
	state: RemoveOrderTransactionState;
};

export class RemoveOrder implements RemoveOrderTransaction {
	private config: Config;
	private chainId: number;
	private subgraphUrl: string;
    private txHash: Hex;
    private onSuccess: () => void;
    private onError: () => void;

	public state: RemoveOrderTransactionState;

	constructor(args: RemoveOrderTransactionArgs, onSuccess: () => void, onError: () => void) {
		this.config = args.config;
		this.chainId = args.chainId;
		this.subgraphUrl = args.subgraphUrl;
        this.txHash = args.txHash;
        this.state = {
            status: TransactionStatusMessage.IDLE,
            message: TransactionStatusMessage.IDLE,
            explorerLink: ''
        };
        this.onSuccess = onSuccess;
        this.onError = onError;
	}

	public get message(): TransactionStatusMessage {
		return this.state.status;
	}

	private updateState(partialState: Partial<RemoveOrderTransactionState>): void {
		this.state = { ...this.state, ...partialState } as RemoveOrderTransactionState;
	}

	public async execute(): Promise<void> {
        const explorerLink = await getExplorerLink(this.txHash, this.chainId, 'tx');
		this.updateState({ 
            status: TransactionStatusMessage.IDLE, 
            message: 'Starting order removal.', 
            explorerLink 
        });
		await this.waitForTxReceipt(this.txHash);
	}

	private async waitForTxReceipt(hash: Hex): Promise<void> {
		try {
			this.updateState({ 
                status: TransactionStatusMessage.PENDING_REMOVE_ORDER,
                message: `Waiting for transaction receipt (hash: ${hash})...` 
            });
            
			await waitForTransactionReceipt(this.config, { hash });
			
            this.updateState({ 
                message: 'Transaction receipt received.' 
            });
            
            await this.indexOrderRemoval(this.txHash);
		} catch (error) {
			this.updateState({
				status: TransactionStatusMessage.ERROR,
				message: 'Failed to get transaction receipt.'
			});
			return this.onError();
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

            await match(result)
                .with({ error: TransactionErrorMessage.TIMEOUT }, () => {
                    this.updateState({
                        status: TransactionStatusMessage.ERROR,
                        message: 'Subgraph indexing timed out.'
                    });
                    return this.onError();
                })
                .with({ value: P.not(P.nullish) }, ({ value }) => {
                    this.updateState({
                        status: TransactionStatusMessage.SUCCESS,
                        hash: value.txHash as Hex,
                        message: value.successMessage || 'Order removal indexed successfully.'
                    });
                    return this.onSuccess();
                })
                .otherwise(() => {
                    this.updateState({
                        status: TransactionStatusMessage.ERROR,
                        message: 'Unknown error during indexing.'
                    });
                    return this.onError();
                });
		} catch (error) {
			this.updateState({
				status: TransactionStatusMessage.ERROR,
				errorDetails: TransactionErrorMessage.SUBGRAPH_FAILED,
				message: 'Failed to index order removal.'
			});
			return this.onError();
		}
	}
}