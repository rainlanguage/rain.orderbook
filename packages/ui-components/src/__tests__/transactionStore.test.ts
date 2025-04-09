import { get } from 'svelte/store';
import { describe, it, expect, vi, beforeEach, afterAll, type Mock } from 'vitest';
import transactionStore, {
	TransactionStatus,
	TransactionErrorMessage
} from '../lib/stores/transactionStore';
import { waitForTransactionReceipt, sendTransaction, switchChain, type Config } from '@wagmi/core';
import {
	getTransaction,
	getTransactionAddOrders,
	getTransactionRemoveOrders
} from '@rainlanguage/orderbook/js_api';
import { getExplorerLink } from '../lib/services/getExplorerLink';
import { waitFor } from '@testing-library/svelte';

vi.mock('@wagmi/core', () => ({
	waitForTransactionReceipt: vi.fn(),
	sendTransaction: vi.fn(),
	switchChain: vi.fn()
}));

vi.mock('@rainlanguage/orderbook/js_api', () => ({
	getTransaction: vi.fn(),
	getTransactionAddOrders: vi.fn(),
	getTransactionRemoveOrders: vi.fn()
}));

vi.mock('../lib/services/getExplorerLink', () => ({
	getExplorerLink: vi.fn()
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
		awaitNewOrderIndexing,
		awaitRemoveOrderIndexing
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
			status: TransactionStatus.IDLE,
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
		expect(get(transactionStore).status).toBe(TransactionStatus.CHECKING_ALLOWANCE);
		expect(get(transactionStore).message).toBe('Checking allowance...');
	});

	it('should update status to PENDING_WALLET', () => {
		awaitWalletConfirmation('Waiting for wallet...');
		expect(get(transactionStore).status).toBe(TransactionStatus.PENDING_WALLET);
		expect(get(transactionStore).message).toBe('Waiting for wallet...');
	});

	it('should update status to PENDING_APPROVAL', () => {
		awaitApprovalTx('mockHash', 'TEST');
		expect(get(transactionStore).status).toBe(TransactionStatus.PENDING_APPROVAL);
		expect(get(transactionStore).hash).toBe('mockHash');
		expect(get(transactionStore).message).toBe('Approving TEST spend...');
	});

	it('should update status to SUCCESS', () => {
		transactionSuccess('mockHash', 'Transaction successful');
		expect(get(transactionStore).status).toBe(TransactionStatus.SUCCESS);
		expect(get(transactionStore).hash).toBe('mockHash');
		expect(get(transactionStore).message).toBe('Transaction successful');
	});

	it('should update status to ERROR', () => {
		transactionError(TransactionErrorMessage.DEPLOY_FAILED, 'mockHash');
		expect(get(transactionStore).status).toBe(TransactionStatus.ERROR);
		expect(get(transactionStore).error).toBe(TransactionErrorMessage.DEPLOY_FAILED);
		expect(get(transactionStore).hash).toBe('mockHash');
	});

	it('should handle successful deployment transaction', async () => {
		const mockApprovals = [
			{ token: '0xtoken1', calldata: '0xapproval1' },
			{ token: '0xtoken2', calldata: '0xapproval2' }
		];
		const mockDeploymentCalldata = '0xdeployment';

		(sendTransaction as Mock).mockResolvedValueOnce('approvalHash1');
		(sendTransaction as Mock).mockResolvedValueOnce('approvalHash2');
		(sendTransaction as Mock).mockResolvedValueOnce('deployHash');
		(getTransaction as Mock).mockReturnValue({ id: 'mockHash' });
		(waitForTransactionReceipt as Mock).mockResolvedValue({});
		(switchChain as Mock).mockResolvedValue({});
		(getExplorerLink as Mock).mockResolvedValue('https://explorer.example.com/tx/deployHash');

		await handleDeploymentTransaction({
			config: mockConfig,
			approvals: mockApprovals,
			deploymentCalldata: mockDeploymentCalldata,
			orderbookAddress: mockOrderbookAddress as `0x${string}`,
			chainId: 1,
			subgraphUrl: 'test.com',
			network: 'flare'
		});

		expect(get(transactionStore).status).toBe(TransactionStatus.PENDING_SUBGRAPH);
		expect(get(transactionStore).hash).toBe('deployHash');
		expect(getExplorerLink).toHaveBeenCalledWith('deployHash', 1, 'tx');
		expect(get(transactionStore).explorerLink).toBe('https://explorer.example.com/tx/deployHash');
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

		expect(get(transactionStore).status).toBe(TransactionStatus.ERROR);
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

		expect(get(transactionStore).status).toBe(TransactionStatus.ERROR);
		expect(get(transactionStore).error).toBe(TransactionErrorMessage.USER_REJECTED_APPROVAL);
	});

	it('should handle approval transaction receipt failure', async () => {
		const mockApprovals = [{ token: '0xtoken1', calldata: '0xapproval1' }];

		(switchChain as Mock).mockResolvedValue({});
		(sendTransaction as Mock).mockResolvedValue('approvalHash');
		(waitForTransactionReceipt as Mock).mockRejectedValue(new Error('Receipt failed'));

		await handleDeploymentTransaction({
			config: mockConfig,
			approvals: mockApprovals,
			deploymentCalldata: '0x',
			orderbookAddress: mockOrderbookAddress as `0x${string}`,
			chainId: 1,
			subgraphUrl: 'test.com',
			network: 'flare'
		});

		expect(get(transactionStore).status).toBe(TransactionStatus.ERROR);
		expect(get(transactionStore).error).toBe(TransactionErrorMessage.APPROVAL_FAILED);
	});

	it('should handle user rejection of deployment transaction', async () => {
		(switchChain as Mock).mockResolvedValue({});
		(sendTransaction as Mock).mockRejectedValue(new Error('User rejected'));

		await handleDeploymentTransaction({
			config: mockConfig,
			approvals: [],
			deploymentCalldata: '0x',
			orderbookAddress: mockOrderbookAddress as `0x${string}`,
			chainId: 1,
			subgraphUrl: 'test.com',
			network: 'flare'
		});

		expect(get(transactionStore).status).toBe(TransactionStatus.ERROR);
		expect(get(transactionStore).error).toBe(TransactionErrorMessage.USER_REJECTED_TRANSACTION);
	});

	it('should handle deployment transaction receipt failure', async () => {
		(switchChain as Mock).mockResolvedValue({});
		(sendTransaction as Mock).mockResolvedValue('deployHash');
		(waitForTransactionReceipt as Mock).mockRejectedValue(new Error('Receipt failed'));

		await handleDeploymentTransaction({
			config: mockConfig,
			approvals: [],
			deploymentCalldata: '0x',
			orderbookAddress: mockOrderbookAddress as `0x${string}`,
			chainId: 1,
			subgraphUrl: 'test.com',
			network: 'flare'
		});

		expect(get(transactionStore).status).toBe(TransactionStatus.ERROR);
		expect(get(transactionStore).error).toBe(TransactionErrorMessage.DEPLOYMENT_FAILED);
	});

	it('should handle multiple approvals successfully', async () => {
		const mockApprovals = [
			{ token: '0xtoken1', calldata: '0xapproval1' },
			{ token: '0xtoken2', calldata: '0xapproval2' }
		];

		(switchChain as Mock).mockResolvedValue({});
		(sendTransaction as Mock)
			.mockResolvedValueOnce('approvalHash1')
			.mockResolvedValueOnce('approvalHash2')
			.mockResolvedValueOnce('deployHash');
		(waitForTransactionReceipt as Mock).mockResolvedValue({});

		await handleDeploymentTransaction({
			config: mockConfig,
			approvals: mockApprovals,
			deploymentCalldata: '0x',
			orderbookAddress: mockOrderbookAddress as `0x${string}`,
			chainId: 1,
			subgraphUrl: 'test.com',
			network: 'flare'
		});

		expect(sendTransaction).toHaveBeenCalledTimes(3); // 2 approvals + 1 deployment
		expect(get(transactionStore).status).toBe(TransactionStatus.PENDING_SUBGRAPH);
	});

	it('should handle successfuly waiting for subgraph indexing', async () => {
		const mockSubgraphUrl = 'test.com';
		const mockTxHash = 'mockHash';
		const mockSuccessMessage = 'Success! Transaction confirmed';

		(getTransaction as Mock).mockResolvedValue({ id: mockTxHash });

		vi.useFakeTimers({ shouldAdvanceTime: true });

		await awaitTransactionIndexing(mockSubgraphUrl, mockTxHash, mockSuccessMessage);

		vi.runOnlyPendingTimers();

		await waitFor(() => {
			expect(get(transactionStore).status).toBe(TransactionStatus.SUCCESS);
			expect(get(transactionStore).message).toBe(mockSuccessMessage);
			expect(get(transactionStore).hash).toBe(mockTxHash);
		});
	});

	it('should handle subgraph indexing timeout', async () => {
		vi.useFakeTimers();
		const mockSubgraphUrl = 'test.com';
		const mockTxHash = 'mockHash';
		const mockSuccessMessage = 'Success message';

		(getTransaction as Mock).mockResolvedValue(null);

		const indexingPromise = awaitTransactionIndexing(
			mockSubgraphUrl,
			mockTxHash,
			mockSuccessMessage
		);

		expect(get(transactionStore).status).toBe(TransactionStatus.PENDING_SUBGRAPH);
		expect(get(transactionStore).message).toBe('Checking for transaction indexing...');

		await vi.advanceTimersByTime(10000);
		await indexingPromise;

		expect(get(transactionStore).message).toBe(
			'The subgraph took too long to respond. Please check again later.'
		);

		vi.useRealTimers();
	});

	it('should handle successful new order indexing', async () => {
		const mockTxHash = 'deployHash';
		const mockSubgraphUrl = 'test.com';
		const mockNetwork = 'flare';
		const mockOrderHash = 'order123';

		(getExplorerLink as Mock).mockResolvedValue(`https://explorer.example.com/tx/${mockTxHash}`);

		(getTransactionAddOrders as Mock).mockResolvedValue([
			{
				order: {
					orderHash: mockOrderHash
				}
			}
		]);

		vi.useFakeTimers({ shouldAdvanceTime: true });

		await awaitNewOrderIndexing(mockSubgraphUrl, mockTxHash, mockNetwork);

		vi.runOnlyPendingTimers();

		await waitFor(() => {
			expect(get(transactionStore).status).toBe(TransactionStatus.SUCCESS);
			expect(get(transactionStore).hash).toBe(mockTxHash);
			expect(get(transactionStore).newOrderHash).toBe(mockOrderHash);
			expect(get(transactionStore).network).toBe(mockNetwork);
		});
	});

	it('should handle new order indexing timeout', async () => {
		vi.useFakeTimers();
		const mockSubgraphUrl = 'test.com';
		const mockTxHash = 'mockHash';
		const mockNetwork = 'flare';

		(getTransactionAddOrders as Mock).mockResolvedValue([]);

		const indexingPromise = awaitNewOrderIndexing(mockSubgraphUrl, mockTxHash, mockNetwork);

		expect(get(transactionStore).status).toBe(TransactionStatus.PENDING_SUBGRAPH);
		expect(get(transactionStore).message).toBe('Waiting for new order to be indexed...');

		await vi.advanceTimersByTime(10000);
		await indexingPromise;

		expect(get(transactionStore).status).toBe(TransactionStatus.ERROR);
		expect(get(transactionStore).message).toBe(
			'The subgraph took too long to respond. Please check again later.'
		);
		expect(get(transactionStore).error).toBe(TransactionErrorMessage.TIMEOUT);

		vi.useRealTimers();
	});

	it('should handle successful remove order indexing', async () => {
		const mockSubgraphUrl = 'test.com';
		const mockTxHash = 'mockHash';

		(getTransactionRemoveOrders as Mock).mockResolvedValue([
			{
				order: {
					id: 'removedOrder123'
				}
			}
		]);

		vi.useFakeTimers({ shouldAdvanceTime: true });

		await awaitRemoveOrderIndexing(mockSubgraphUrl, mockTxHash);

		vi.runOnlyPendingTimers();

		await waitFor(() => {
			expect(get(transactionStore).status).toBe(TransactionStatus.SUCCESS);
			expect(get(transactionStore).hash).toBe(mockTxHash);
		});
	});

	it('should handle remove order indexing timeout', async () => {
		vi.useFakeTimers();
		const mockSubgraphUrl = 'test.com';
		const mockTxHash = 'mockHash';

		(getTransactionRemoveOrders as Mock).mockResolvedValue([]);

		const indexingPromise = awaitRemoveOrderIndexing(mockSubgraphUrl, mockTxHash);

		expect(get(transactionStore).status).toBe(TransactionStatus.PENDING_SUBGRAPH);
		expect(get(transactionStore).message).toBe('Waiting for order removal to be indexed...');

		await vi.advanceTimersByTime(10000);
		await indexingPromise;

		expect(get(transactionStore).status).toBe(TransactionStatus.ERROR);
		expect(get(transactionStore).message).toBe(
			'The subgraph took too long to respond. Please check again later.'
		);
		expect(get(transactionStore).error).toBe(TransactionErrorMessage.TIMEOUT);

		vi.useRealTimers();
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
			subgraphUrl: '', // Empty subgraphUrl
			network: 'flare'
		});

		expect(get(transactionStore).status).toBe(TransactionStatus.SUCCESS);
		expect(get(transactionStore).hash).toBe('deployHash');
		expect(get(transactionStore).message).toBe(
			'Deployment successful. Check the Orders page for your new order.'
		);
		expect(get(transactionStore).network).toBe('flare');
		expect(getExplorerLink).toHaveBeenCalledWith('deployHash', 1, 'tx');
		expect(get(transactionStore).explorerLink).toBe('https://explorer.example.com/tx/deployHash');

		expect(getTransactionAddOrders).not.toHaveBeenCalled();
	});
});

describe('handleRemoveOrderTransaction', () => {
	const mockConfig = {} as Config;
	const mockOrderbookAddress = '0xabcdef1234567890' as `0x${string}`;
	const mockRemoveOrderCalldata = '0xremovecalldata';
	const mockChainId = 1;
	const mockSubgraphUrl = 'https://api.thegraph.com/subgraphs/name/test/orderbook';

	const { reset, handleRemoveOrderTransaction } = transactionStore;

	beforeEach(() => {
		vi.resetAllMocks();
		reset();
	});

	afterAll(() => {
		vi.clearAllMocks();
	});

	it('should successfully handle a remove order transaction', async () => {
		(switchChain as Mock).mockResolvedValue({});

		const mockTxHash = '0xremoveordertxhash';
		(sendTransaction as Mock).mockResolvedValue(mockTxHash);
		vi.useFakeTimers({ shouldAdvanceTime: true });

		(waitForTransactionReceipt as Mock).mockResolvedValue({});
		(getTransactionRemoveOrders as Mock).mockResolvedValue([{ order: { id: 'removedOrder123' } }]);

		await handleRemoveOrderTransaction({
			config: mockConfig,
			orderbookAddress: mockOrderbookAddress,
			removeOrderCalldata: mockRemoveOrderCalldata,
			chainId: mockChainId,
			subgraphUrl: mockSubgraphUrl
		});

		const state = get(transactionStore);
		expect(state.status).toBe(TransactionStatus.PENDING_SUBGRAPH);
		expect(state.hash).toBe(mockTxHash);
		expect(state.message).toBe('Waiting for order removal to be indexed...');

		expect(switchChain).toHaveBeenCalledWith(mockConfig, { chainId: mockChainId });
		expect(sendTransaction).toHaveBeenCalledWith(mockConfig, {
			to: mockOrderbookAddress,
			data: mockRemoveOrderCalldata
		});
		expect(waitForTransactionReceipt).toHaveBeenCalledWith(mockConfig, { hash: mockTxHash });

		vi.runOnlyPendingTimers();

		await waitFor(() => {
			expect(get(transactionStore).status).toBe(TransactionStatus.SUCCESS);
			expect(get(transactionStore).hash).toBe(mockTxHash);
		});
	});

	it('should skip subgraph indexing when subgraphUrl is not provided', async () => {
		(switchChain as Mock).mockResolvedValue({});

		const mockTxHash = '0xremoveordertxhash';
		(sendTransaction as Mock).mockResolvedValue(mockTxHash);

		(waitForTransactionReceipt as Mock).mockResolvedValue({});

		await handleRemoveOrderTransaction({
			config: mockConfig,
			orderbookAddress: mockOrderbookAddress,
			removeOrderCalldata: mockRemoveOrderCalldata,
			chainId: mockChainId,
			subgraphUrl: ''
		});

		const state = get(transactionStore);
		expect(state.status).toBe(TransactionStatus.PENDING_SUBGRAPH);
		expect(state.hash).toBe(mockTxHash);
		expect(state.message).toBe('Waiting for order removal to be indexed...');

		expect(switchChain).toHaveBeenCalledWith(mockConfig, { chainId: mockChainId });
		expect(sendTransaction).toHaveBeenCalledWith(mockConfig, {
			to: mockOrderbookAddress,
			data: mockRemoveOrderCalldata
		});
		expect(waitForTransactionReceipt).toHaveBeenCalledWith(mockConfig, { hash: mockTxHash });
	});

	it('should handle chain switch failure', async () => {
		(switchChain as Mock).mockRejectedValue(new Error('Failed to switch chain'));

		await handleRemoveOrderTransaction({
			config: mockConfig,
			orderbookAddress: mockOrderbookAddress,
			removeOrderCalldata: mockRemoveOrderCalldata,
			chainId: mockChainId,
			subgraphUrl: mockSubgraphUrl
		});

		const state = get(transactionStore);
		expect(state.status).toBe(TransactionStatus.ERROR);
		expect(state.error).toBe(TransactionErrorMessage.SWITCH_CHAIN_FAILED);

		expect(switchChain).toHaveBeenCalledWith(mockConfig, { chainId: mockChainId });
		expect(sendTransaction).not.toHaveBeenCalled();
	});

	it('should handle user rejection of transaction', async () => {
		(switchChain as Mock).mockResolvedValue({});

		(sendTransaction as Mock).mockRejectedValue(new Error('User denied transaction'));

		await handleRemoveOrderTransaction({
			config: mockConfig,
			orderbookAddress: mockOrderbookAddress,
			removeOrderCalldata: mockRemoveOrderCalldata,
			chainId: mockChainId,
			subgraphUrl: mockSubgraphUrl
		});

		const state = get(transactionStore);
		expect(state.status).toBe(TransactionStatus.ERROR);
		expect(state.error).toBe(TransactionErrorMessage.USER_REJECTED_TRANSACTION);

		expect(switchChain).toHaveBeenCalledWith(mockConfig, { chainId: mockChainId });
		expect(sendTransaction).toHaveBeenCalledWith(mockConfig, {
			to: mockOrderbookAddress,
			data: mockRemoveOrderCalldata
		});
		expect(waitForTransactionReceipt).not.toHaveBeenCalled();
	});

	it('should handle transaction receipt failure', async () => {
		(switchChain as Mock).mockResolvedValue({});

		const mockTxHash = '0xremoveordertxhash';
		(sendTransaction as Mock).mockResolvedValue(mockTxHash);

		(waitForTransactionReceipt as Mock).mockRejectedValue(new Error('Transaction failed'));

		await handleRemoveOrderTransaction({
			config: mockConfig,
			orderbookAddress: mockOrderbookAddress,
			removeOrderCalldata: mockRemoveOrderCalldata,
			chainId: mockChainId,
			subgraphUrl: mockSubgraphUrl
		});

		const state = get(transactionStore);
		expect(state.status).toBe(TransactionStatus.ERROR);
		expect(state.error).toBe(TransactionErrorMessage.REMOVE_ORDER_FAILED);

		expect(switchChain).toHaveBeenCalledWith(mockConfig, { chainId: mockChainId });
		expect(sendTransaction).toHaveBeenCalledWith(mockConfig, {
			to: mockOrderbookAddress,
			data: mockRemoveOrderCalldata
		});
		expect(waitForTransactionReceipt).toHaveBeenCalledWith(mockConfig, { hash: mockTxHash });
	});

	it('should handle subgraph indexing timeout', async () => {
		vi.useFakeTimers();
		(switchChain as Mock).mockResolvedValue({});

		const mockTxHash = '0xremoveordertxhash';
		(sendTransaction as Mock).mockResolvedValue(mockTxHash);

		(waitForTransactionReceipt as Mock).mockResolvedValue({});

		(getTransactionRemoveOrders as Mock).mockImplementation(() => {
			return Promise.resolve([]);
		});

		const promise = handleRemoveOrderTransaction({
			config: mockConfig,
			orderbookAddress: mockOrderbookAddress,
			removeOrderCalldata: mockRemoveOrderCalldata,
			chainId: mockChainId,
			subgraphUrl: mockSubgraphUrl
		});

		await promise;

		await vi.advanceTimersByTime(10000);

		const state = get(transactionStore);
		expect(state.status).toBe(TransactionStatus.ERROR);
		expect(state.error).toBe(TransactionErrorMessage.TIMEOUT);
		expect(state.message).toBe('The subgraph took too long to respond. Please check again later.');

		vi.useRealTimers();
	});
});
