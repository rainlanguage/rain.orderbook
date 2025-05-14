import type { Hex } from 'viem';
import { waitForTransactionReceipt } from '@wagmi/core';
import { TransactionStatusMessage, TransactionErrorMessage } from '$lib/types/transaction';
import type { TransactionArgs } from '$lib/types/transaction';
import {
	awaitSubgraphIndexing,
	getRemoveOrderConfig
} from '$lib/services/awaitTransactionIndexing';
import type { Config } from '@wagmi/core';
import { getExplorerLink } from '$lib/services/getExplorerLink';
import { match, P } from 'ts-pattern';
import { writable, type Writable, get } from 'svelte/store';

export type RemoveOrderTransactionState = 
    | { status: TransactionStatusMessage.IDLE; message: string; explorerLink: string }
    | { status: TransactionStatusMessage.PENDING_REMOVE_ORDER; message: string; explorerLink: string }
    | { status: TransactionStatusMessage.PENDING_SUBGRAPH; message: string; explorerLink: string }
    | { status: TransactionStatusMessage.SUCCESS; message: string; explorerLink: string; hash?: Hex }
    | { status: TransactionStatusMessage.ERROR; message: string; explorerLink: string; errorDetails?: TransactionErrorMessage };

export type RemoveOrderTransaction = {
	readonly state: Writable<RemoveOrderTransactionState>;
};

export class RemoveOrder implements RemoveOrderTransaction {
	private config: Config;
	private chainId: number;
	private subgraphUrl: string;
    private txHash: Hex;
    private onSuccess: () => void;
    private onError: () => void;

	public readonly state: Writable<RemoveOrderTransactionState>;

	constructor(args: TransactionArgs, onSuccess: () => void, onError: () => void) {
		console.log('RemoveOrder: Initializing with transaction hash', args.txHash);
		this.config = args.config;
		this.chainId = args.chainId;
		this.subgraphUrl = args.subgraphUrl;
        this.txHash = args.txHash;
        this.state = writable<RemoveOrderTransactionState>({
            status: TransactionStatusMessage.IDLE,
            message: '',
            explorerLink: ''
        });
        this.onSuccess = onSuccess;
        this.onError = onError;
		console.log('RemoveOrder: Initialization complete');
	}

	public get message(): string {
		return get(this.state).message;
	}

	private updateState(partialState: Partial<RemoveOrderTransactionState>): void {
		console.log('RemoveOrder: Updating state', partialState);
		this.state.update(currentState => ({
            ...currentState,
            ...partialState
        } as RemoveOrderTransactionState));
	}

	public async execute(): Promise<void> {
		console.log('RemoveOrder: Starting execution for hash', this.txHash);
        const explorerLink = await getExplorerLink(this.txHash, this.chainId, 'tx');
		console.log('RemoveOrder: Generated explorer link', explorerLink);
		this.updateState({ 
            status: TransactionStatusMessage.IDLE, 
            message: 'Starting order removal.',
            explorerLink 
        });
		await this.waitForTxReceipt(this.txHash);
	}

	private async waitForTxReceipt(hash: Hex): Promise<void> {
		console.log('RemoveOrder: Waiting for transaction receipt', hash);
		try {
			this.updateState({ 
                status: TransactionStatusMessage.PENDING_REMOVE_ORDER,
                message: `Waiting for transaction receipt...` 
            });
            
			console.log('RemoveOrder: Calling waitForTransactionReceipt');
			await waitForTransactionReceipt(this.config, { hash });
			console.log('RemoveOrder: Transaction receipt received');
			
            this.updateState({ 
                message: 'Transaction receipt received.' 
            });
            
            this.indexOrderRemoval(this.txHash);
		} catch (error) {
			console.error('RemoveOrder: Failed to get transaction receipt', error);
			this.updateState({
				status: TransactionStatusMessage.ERROR,
				message: 'Failed to get transaction receipt.'
			});
			return this.onError();
		}
	}

	private async indexOrderRemoval(txHash: Hex): Promise<void> {
		console.log('RemoveOrder: Starting to index order removal', txHash);
		this.updateState({
			status: TransactionStatusMessage.PENDING_SUBGRAPH,
			message: 'Waiting for order removal to be indexed...'
		});
        
		try {
			console.log('RemoveOrder: Calling awaitSubgraphIndexing with URL', this.subgraphUrl);
			const result = await awaitSubgraphIndexing(
				getRemoveOrderConfig(this.subgraphUrl, txHash, 'Order removed successfully')
			);
			console.log('RemoveOrder: Indexing result received', result);

            await match(result)
                .with({ error: TransactionErrorMessage.TIMEOUT }, () => {
                    console.error('RemoveOrder: Subgraph indexing timed out');
                    this.updateState({
                        status: TransactionStatusMessage.ERROR,
                        message: 'Subgraph indexing timed out.'
                    });
                    return this.onError();
                })
                .with({ value: P.not(P.nullish) }, ({ value }) => {
                    console.log('RemoveOrder: Order removal indexed successfully', value);
                    this.updateState({
                        status: TransactionStatusMessage.SUCCESS,
                        hash: value.txHash as Hex,
                        message: 'Order removal indexed successfully.'
                    });
                    return this.onSuccess();
                })
                .otherwise(() => {
                    console.error('RemoveOrder: Unknown error during indexing');
                    this.updateState({
                        status: TransactionStatusMessage.ERROR,
                        message: 'Unknown error during indexing.'
                    });
                    return this.onError();
                });
		} catch (error) {
			console.error('RemoveOrder: Failed to index order removal', error);
			this.updateState({
				status: TransactionStatusMessage.ERROR,
				errorDetails: TransactionErrorMessage.SUBGRAPH_FAILED,
				message: 'Failed to index order removal.'
			});
			return this.onError();
		}
	}
}