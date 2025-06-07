import { get } from 'svelte/store';
import { describe, it, expect, vi, beforeEach, afterAll, type Mock } from 'vitest';
import { TransactionStatusMessage } from '../lib/types/transaction';
import { TransactionErrorMessage } from '../lib/stores/transactionStore';
import transactionStore from '../lib/stores/transactionStore';
import { waitForTransactionReceipt, sendTransaction, switchChain, type Config } from '@wagmi/core';
import { getTransaction } from '@rainlanguage/orderbook';
import { getExplorerLink } from '../lib/services/getExplorerLink';

import { awaitSubgraphIndexing } from '../lib/services/awaitTransactionIndexing';

vi.mock('@wagmi/core', () => ({
	waitForTransactionReceipt: vi.fn(),
	sendTransaction: vi.fn(),
	switchChain: vi.fn()
}));

vi.mock('@rainlanguage/orderbook', () => ({
	getTransaction: vi.fn(),
	getTransactionAddOrders: vi.fn()
}));

vi.mock('../lib/services/getExplorerLink', () => ({
	getExplorerLink: vi.fn()
}));

vi.mock('../lib/services/awaitTransactionIndexing', () => ({
	awaitSubgraphIndexing: vi.fn()
}));

describe('transactionStore', () => {
	const mockConfig = {} as Config;
	const mockOrderbookAddress = '0xabcdef1234567890';

	const {
		reset,
		checkingWalletAllowance,
		handleDeploymentTransaction,
		awaitWalletConfirmation,
		awaitApprovalTx,
		transactionSuccess,
		transactionError,
		awaitTransactionIndexing,
		awaitNewOrderIndexing
	} = transactionStore;

	beforeEach(() => {
		vi.resetAllMocks();
		reset();
	});

	afterAll(() => {
		vi.clearAllMocks();
	});

	it('should initialize with the correct default state', () => {
		expect(get(transactionStore)).toEqual({
			status: TransactionStatusMessage.IDLE,
			error: '',
			hash: '',
			data: null,
			functionName: '',
			message: '',
			newOrderHash: '',
			network: '',
			explorerLink: ''
		});
	});

	it('should update status to CHECKING_ALLOWANCE', () => {
		checkingWalletAllowance('Checking allowance...');
		expect(get(transactionStore).status).toBe(TransactionStatusMessage.CHECKING_ALLOWANCE);
		expect(get(transactionStore).message).toBe('Checking allowance...');
	});

	it('should update status to PENDING_WALLET', () => {
		awaitWalletConfirmation('Waiting for wallet...');
		expect(get(transactionStore).status).toBe(TransactionStatusMessage.PENDING_WALLET);
		expect(get(transactionStore).message).toBe('Waiting for wallet...');
	});

	it('should update status to PENDING_APPROVAL', () => {
		awaitApprovalTx('mockHash', 'TEST');
		expect(get(transactionStore).status).toBe(TransactionStatusMessage.PENDING_APPROVAL);
		expect(get(transactionStore).hash).toBe('mockHash');
		expect(get(transactionStore).message).toBe('Approving TEST spend...');
	});

	it('should update status to SUCCESS', () => {
		transactionSuccess('mockHash', 'Transaction successful');
		expect(get(transactionStore).status).toBe(TransactionStatusMessage.SUCCESS);
		expect(get(transactionStore).hash).toBe('mockHash');
		expect(get(transactionStore).message).toBe('Transaction successful');
	});

	it('should update status to ERROR', () => {
		transactionError(TransactionErrorMessage.DEPLOYMENT_FAILED, 'mockHash');
		expect(get(transactionStore).status).toBe(TransactionStatusMessage.ERROR);
		expect(get(transactionStore).error).toBe(TransactionErrorMessage.DEPLOYMENT_FAILED);
		expect(get(transactionStore).hash).toBe('mockHash');
	});

	it('should handle chain switch failure', async () => {
		(switchChain as Mock).mockRejectedValue(new Error('Switch failed'));

		await handleDeploymentTransaction({
			config: mockConfig,
			approvals: [],
			deploymentCalldata: '0x',
			orderbookAddress: mockOrderbookAddress as `0x${string}`,
			chainId: 1,
			subgraphUrl: 'test.com',
			network: 'flare'
		});

		expect(get(transactionStore).status).toBe(TransactionStatusMessage.ERROR);
		expect(get(transactionStore).error).toBe(TransactionErrorMessage.SWITCH_CHAIN_FAILED);
	});

	it('should handle user rejection of approval transaction', async () => {
		const mockApprovals = [{ token: '0xtoken1', calldata: '0xapproval1' }];

		(switchChain as Mock).mockResolvedValue({});
		(sendTransaction as Mock).mockRejectedValue(new Error('User rejected'));

		await handleDeploymentTransaction({
			config: mockConfig,
			approvals: mockApprovals,
			deploymentCalldata: '0x',
			orderbookAddress: mockOrderbookAddress as `0x${string}`,
			chainId: 1,
			subgraphUrl: 'test.com',
			network: 'flare'
		});

		expect(get(transactionStore).status).toBe(TransactionStatusMessage.ERROR);
		expect(get(transactionStore).error).toBe(TransactionErrorMessage.USER_REJECTED_APPROVAL);
	});

	it('should handle successful transaction indexing', async () => {
		const mockSubgraphUrl = 'test.com';
		const mockTxHash = 'mockHash';
		const mockSuccessMessage = 'Success! Transaction confirmed';

		(awaitSubgraphIndexing as Mock).mockResolvedValue({
			value: {
				txHash: mockTxHash,
				successMessage: mockSuccessMessage
			}
		});

		await awaitTransactionIndexing(mockSubgraphUrl, mockTxHash, mockSuccessMessage);

		expect(awaitSubgraphIndexing).toHaveBeenCalled();

		expect(get(transactionStore).status).toBe(TransactionStatusMessage.SUCCESS);
		expect(get(transactionStore).hash).toBe(mockTxHash);
		expect(get(transactionStore).message).toBe(mockSuccessMessage);
	});

	it('should handle transaction indexing error', async () => {
		const mockSubgraphUrl = 'test.com';
		const mockTxHash = 'mockHash';
		const mockSuccessMessage = 'Success message';

		(awaitSubgraphIndexing as Mock).mockResolvedValue({
			error: TransactionErrorMessage.TIMEOUT
		});

		await awaitTransactionIndexing(mockSubgraphUrl, mockTxHash, mockSuccessMessage);

		expect(get(transactionStore).status).toBe(TransactionStatusMessage.ERROR);
		expect(get(transactionStore).error).toBe(TransactionErrorMessage.TIMEOUT);
	});

	it('should handle successful new order indexing', async () => {
		const mockTxHash = 'deployHash';
		const mockSubgraphUrl = 'test.com';
		const mockNetwork = 'flare';
		const mockOrderHash = 'order123';

		(awaitSubgraphIndexing as Mock).mockResolvedValue({
			value: {
				txHash: mockTxHash,
				successMessage: '',
				orderHash: mockOrderHash,
				network: mockNetwork
			}
		});

		await awaitNewOrderIndexing(mockSubgraphUrl, mockTxHash, mockNetwork);

		expect(awaitSubgraphIndexing).toHaveBeenCalled();
		expect(get(transactionStore).status).toBe(TransactionStatusMessage.SUCCESS);
		expect(get(transactionStore).hash).toBe(mockTxHash);
		expect(get(transactionStore).newOrderHash).toBe(mockOrderHash);
		expect(get(transactionStore).network).toBe(mockNetwork);
	});

	it('should handle new order indexing error', async () => {
		const mockSubgraphUrl = 'test.com';
		const mockTxHash = 'mockHash';
		const mockNetwork = 'flare';

		(awaitSubgraphIndexing as Mock).mockResolvedValue({
			error: TransactionErrorMessage.TIMEOUT
		});

		await awaitNewOrderIndexing(mockSubgraphUrl, mockTxHash, mockNetwork);

		expect(get(transactionStore).status).toBe(TransactionStatusMessage.ERROR);
		expect(get(transactionStore).error).toBe(TransactionErrorMessage.TIMEOUT);
	});

	it('should skip subgraph indexing when subgraphUrl is not provided', async () => {
		(switchChain as Mock).mockResolvedValue({});
		(sendTransaction as Mock).mockResolvedValue('deployHash');
		(waitForTransactionReceipt as Mock).mockResolvedValue({});
		(getExplorerLink as Mock).mockResolvedValue('https://explorer.example.com/tx/deployHash');

		await handleDeploymentTransaction({
			config: mockConfig,
			approvals: [],
			deploymentCalldata: '0xdeployment',
			orderbookAddress: mockOrderbookAddress as `0x${string}`,
			chainId: 1,
			subgraphUrl: '',
			network: 'flare'
		});

		expect(get(transactionStore).status).toBe(TransactionStatusMessage.SUCCESS);
		expect(get(transactionStore).hash).toBe('deployHash');
		expect(get(transactionStore).message).toBe(
			'Deployment successful. Check the Orders page for your new order.'
		);
		expect(get(transactionStore).network).toBe('flare');
		expect(getExplorerLink).toHaveBeenCalledWith('deployHash', 1, 'tx');
		expect(get(transactionStore).explorerLink).toBe('https://explorer.example.com/tx/deployHash');

		expect(awaitSubgraphIndexing).not.toHaveBeenCalled();
	});
});

describe('handleDeploymentTransaction', () => {
	const mockConfig = {} as Config;
	const mockOrderbookAddress = '0xabcdef1234567890' as `0x${string}`;
	const mockChainId = 1;
	const mockSubgraphUrl = 'https://api.thegraph.com/subgraphs/name/test/orderbook';
	const mockApprovals = [
		{ token: '0xtoken1', calldata: '0xapproval1' },
		{ token: '0xtoken2', calldata: '0xapproval2' }
	];
	const mockDeploymentCalldata = '0xdeployment';

	const { reset, handleDeploymentTransaction } = transactionStore;

	beforeEach(() => {
		vi.resetAllMocks();
		reset();
	});

	afterAll(() => {
		vi.clearAllMocks();
	});

	it('should handle successful deployment transaction', async () => {
		(sendTransaction as Mock).mockResolvedValueOnce('approvalHash1');
		(sendTransaction as Mock).mockResolvedValueOnce('approvalHash2');
		(sendTransaction as Mock).mockResolvedValueOnce('deployHash');
		(getTransaction as Mock).mockReturnValue({ id: 'mockHash' });
		(waitForTransactionReceipt as Mock).mockResolvedValue({});
		(switchChain as Mock).mockResolvedValue({});
		(getExplorerLink as Mock).mockResolvedValue('https://explorer.example.com/tx/deployHash');

		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		let resolveIndexing: ((value: unknown) => void) | undefined;
		const indexingPromise = new Promise<unknown>((resolve) => {
			resolveIndexing = resolve;
		});

		(awaitSubgraphIndexing as Mock).mockReturnValue(indexingPromise);

		const deploymentPromise = handleDeploymentTransaction({
			config: mockConfig,
			approvals: mockApprovals,
			deploymentCalldata: mockDeploymentCalldata,
			orderbookAddress: mockOrderbookAddress as `0x${string}`,
			chainId: 1,
			subgraphUrl: 'test.com',
			network: 'flare'
		});

		await new Promise((resolve) => setTimeout(resolve, 0));

		const pendingState = get(transactionStore);
		expect(pendingState.status).toBe(TransactionStatusMessage.PENDING_SUBGRAPH);
		expect(pendingState.hash).toBe('deployHash');
		expect(pendingState.explorerLink).toBe('https://explorer.example.com/tx/deployHash');

		expect(getExplorerLink).toHaveBeenCalledWith('deployHash', 1, 'tx');

		if (resolveIndexing) {
			resolveIndexing({
				value: {
					txHash: 'mockHash',
					successMessage: 'Transaction confirmed'
				}
			});
		}

		await deploymentPromise;

		const finalState = get(transactionStore);
		expect(finalState.status).toBe(TransactionStatusMessage.SUCCESS);
		expect(finalState.message).toBe('Transaction confirmed');
		expect(finalState.explorerLink).toBe('https://explorer.example.com/tx/deployHash');
	});
	it('should handle subgraph indexing error', async () => {
		(switchChain as Mock).mockResolvedValue({});

		const mockTxHash = '0xdeposittxhash';
		(sendTransaction as Mock).mockResolvedValue(mockTxHash);

		(waitForTransactionReceipt as Mock).mockResolvedValue({});
		(getExplorerLink as Mock).mockResolvedValue('https://explorer.example.com/tx/deployHash');

		(awaitSubgraphIndexing as Mock).mockResolvedValue({
			error: TransactionErrorMessage.TIMEOUT
		});

		await handleDeploymentTransaction({
			config: mockConfig,
			approvals: mockApprovals,
			deploymentCalldata: mockDeploymentCalldata,
			orderbookAddress: mockOrderbookAddress as `0x${string}`,
			chainId: mockChainId,
			subgraphUrl: mockSubgraphUrl,
			network: 'flare'
		});

		const state = get(transactionStore);
		expect(state.status).toBe(TransactionStatusMessage.ERROR);
		expect(state.error).toBe(TransactionErrorMessage.TIMEOUT);
	});
});
