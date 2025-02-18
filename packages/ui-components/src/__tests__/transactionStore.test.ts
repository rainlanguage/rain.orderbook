import { describe, it, expect, vi, beforeEach, afterAll, type Mock } from 'vitest';
import { get } from 'svelte/store';
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
			newOrderId: '',
			network: ''
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
		const mockSubgraphUrl = 'test.com';
		const mockTxHash = 'mockHash';
		const mockNetwork = 'flare';
		const mockOrderId = 'order123';

		(getTransactionAddOrders as Mock).mockResolvedValue([
			{
				order: {
					id: mockOrderId
				}
			}
		]);

		vi.useFakeTimers({ shouldAdvanceTime: true });

		await awaitNewOrderIndexing(mockSubgraphUrl, mockTxHash, mockNetwork);

		vi.runOnlyPendingTimers();

		await waitFor(() => {
			expect(get(transactionStore).status).toBe(TransactionStatus.SUCCESS);
			expect(get(transactionStore).hash).toBe(mockTxHash);
			expect(get(transactionStore).newOrderId).toBe(mockOrderId);
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
		expect(get(transactionStore).message).toBe('Waiting for new Order to be indexed...');

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
		const mockNetwork = 'flare';

		(getTransactionRemoveOrders as Mock).mockResolvedValue([
			{
				order: {
					id: 'removedOrder123'
				}
			}
		]);

		vi.useFakeTimers({ shouldAdvanceTime: true });

		await awaitRemoveOrderIndexing(mockSubgraphUrl, mockTxHash, mockNetwork);

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
		const mockNetwork = 'flare';

		(getTransactionRemoveOrders as Mock).mockResolvedValue([]);

		const indexingPromise = awaitRemoveOrderIndexing(mockSubgraphUrl, mockTxHash, mockNetwork);

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
});
