import { get } from 'svelte/store';
import { describe, it, expect, vi, beforeEach, afterAll, type Mock } from 'vitest';
import transactionStore, {
	TransactionStatus,
	TransactionErrorMessage
} from '../lib/stores/transactionStore';
import { waitForTransactionReceipt, sendTransaction, switchChain, type Config } from '@wagmi/core';
import {
	getTransaction,
	type DepositCalldataResult,
	type SgVault,
	type WithdrawCalldataResult
} from '@rainlanguage/orderbook';
import { getExplorerLink } from '../lib/services/getExplorerLink';

import {
	awaitSubgraphIndexing,
	getTransactionConfig,
	getNewOrderConfig,
	getRemoveOrderConfig,
	TIMEOUT_ERROR
} from '../lib/services/awaitTransactionIndexing';

vi.mock('@wagmi/core', () => ({
	waitForTransactionReceipt: vi.fn(),
	sendTransaction: vi.fn(),
	switchChain: vi.fn()
}));

vi.mock('@rainlanguage/orderbook', () => ({
	getTransaction: vi.fn(),
	getTransactionAddOrders: vi.fn(),
	getTransactionRemoveOrders: vi.fn()
}));

vi.mock('../lib/services/getExplorerLink', () => ({
	getExplorerLink: vi.fn()
}));

vi.mock('../lib/services/awaitTransactionIndexing', () => ({
	awaitSubgraphIndexing: vi.fn(),
	getTransactionConfig: vi.fn(),
	getNewOrderConfig: vi.fn(),
	getRemoveOrderConfig: vi.fn(),
	TIMEOUT_ERROR: 'The subgraph took too long to respond. Please check the transaction link.'
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
		expect(pendingState.status).toBe(TransactionStatus.PENDING_SUBGRAPH);
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
		expect(finalState.status).toBe(TransactionStatus.SUCCESS);
		expect(finalState.message).toBe('Transaction confirmed');
		expect(finalState.explorerLink).toBe('https://explorer.example.com/tx/deployHash');
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

		(getTransactionConfig as Mock).mockReturnValue({
			subgraphUrl: mockSubgraphUrl,
			txHash: mockTxHash,
			successMessage: mockSuccessMessage
		});

		await awaitTransactionIndexing(mockSubgraphUrl, mockTxHash, mockSuccessMessage);

		expect(awaitSubgraphIndexing).toHaveBeenCalled();
		expect(getTransactionConfig).toHaveBeenCalledWith(
			mockSubgraphUrl,
			mockTxHash,
			mockSuccessMessage
		);

		expect(get(transactionStore).status).toBe(TransactionStatus.SUCCESS);
		expect(get(transactionStore).hash).toBe(mockTxHash);
		expect(get(transactionStore).message).toBe(mockSuccessMessage);
	});

	it('should handle transaction indexing error', async () => {
		const mockSubgraphUrl = 'test.com';
		const mockTxHash = 'mockHash';
		const mockSuccessMessage = 'Success message';

		(awaitSubgraphIndexing as Mock).mockResolvedValue({
			error: TIMEOUT_ERROR
		});

		(getTransactionConfig as Mock).mockReturnValue({
			subgraphUrl: mockSubgraphUrl,
			txHash: mockTxHash,
			successMessage: mockSuccessMessage
		});

		await awaitTransactionIndexing(mockSubgraphUrl, mockTxHash, mockSuccessMessage);

		expect(get(transactionStore).status).toBe(TransactionStatus.ERROR);
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

		(getNewOrderConfig as Mock).mockReturnValue({
			subgraphUrl: mockSubgraphUrl,
			txHash: mockTxHash,
			successMessage: '',
			network: mockNetwork
		});

		await awaitNewOrderIndexing(mockSubgraphUrl, mockTxHash, mockNetwork);

		expect(awaitSubgraphIndexing).toHaveBeenCalled();
		expect(getNewOrderConfig).toHaveBeenCalledWith(mockSubgraphUrl, mockTxHash, '', mockNetwork);

		expect(get(transactionStore).status).toBe(TransactionStatus.SUCCESS);
		expect(get(transactionStore).hash).toBe(mockTxHash);
		expect(get(transactionStore).newOrderHash).toBe(mockOrderHash);
		expect(get(transactionStore).network).toBe(mockNetwork);
	});

	it('should handle new order indexing error', async () => {
		const mockSubgraphUrl = 'test.com';
		const mockTxHash = 'mockHash';
		const mockNetwork = 'flare';

		(awaitSubgraphIndexing as Mock).mockResolvedValue({
			error: TIMEOUT_ERROR
		});

		(getNewOrderConfig as Mock).mockReturnValue({
			subgraphUrl: mockSubgraphUrl,
			txHash: mockTxHash,
			successMessage: '',
			network: mockNetwork
		});

		await awaitNewOrderIndexing(mockSubgraphUrl, mockTxHash, mockNetwork);

		expect(get(transactionStore).status).toBe(TransactionStatus.ERROR);
		expect(get(transactionStore).error).toBe(TransactionErrorMessage.TIMEOUT);
	});

	it('should handle successful remove order indexing', async () => {
		const mockSubgraphUrl = 'test.com';
		const mockTxHash = 'mockHash';
		const mockSuccessMessage = 'Order removed successfully';

		(awaitSubgraphIndexing as Mock).mockResolvedValue({
			value: {
				txHash: mockTxHash,
				successMessage: mockSuccessMessage
			}
		});

		(getRemoveOrderConfig as Mock).mockReturnValue({
			subgraphUrl: mockSubgraphUrl,
			txHash: mockTxHash,
			successMessage: mockSuccessMessage
		});

		await awaitRemoveOrderIndexing(mockSubgraphUrl, mockTxHash);

		expect(awaitSubgraphIndexing).toHaveBeenCalled();
		expect(getRemoveOrderConfig).toHaveBeenCalledWith(
			mockSubgraphUrl,
			mockTxHash,
			'Order removed successfully'
		);

		expect(get(transactionStore).status).toBe(TransactionStatus.SUCCESS);
		expect(get(transactionStore).hash).toBe(mockTxHash);
		expect(get(transactionStore).message).toBe(mockSuccessMessage);
	});

	it('should handle remove order indexing error', async () => {
		const mockSubgraphUrl = 'test.com';
		const mockTxHash = 'mockHash';

		(awaitSubgraphIndexing as Mock).mockResolvedValue({
			error: TIMEOUT_ERROR
		});

		(getRemoveOrderConfig as Mock).mockReturnValue({
			subgraphUrl: mockSubgraphUrl,
			txHash: mockTxHash,
			successMessage: 'Order removed successfully'
		});

		await awaitRemoveOrderIndexing(mockSubgraphUrl, mockTxHash);

		expect(get(transactionStore).status).toBe(TransactionStatus.ERROR);
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

		expect(get(transactionStore).status).toBe(TransactionStatus.SUCCESS);
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
		(waitForTransactionReceipt as Mock).mockResolvedValue({});
		(getExplorerLink as Mock).mockResolvedValue(
			'https://explorer.example.com/tx/removeordertxhash'
		);

		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		let resolveIndexing: ((value: unknown) => void) | undefined;
		const indexingPromise = new Promise<unknown>((resolve) => {
			resolveIndexing = resolve;
		});

		(awaitSubgraphIndexing as Mock).mockReturnValue(indexingPromise);

		const transactionPromise = handleRemoveOrderTransaction({
			config: mockConfig,
			orderbookAddress: mockOrderbookAddress,
			removeOrderCalldata: mockRemoveOrderCalldata,
			chainId: mockChainId,
			subgraphUrl: mockSubgraphUrl
		});

		await new Promise((resolve) => setTimeout(resolve, 0));

		const pendingState = get(transactionStore);
		expect(pendingState.status).toBe(TransactionStatus.PENDING_SUBGRAPH);
		expect(pendingState.hash).toBe(mockTxHash);
		expect(pendingState.message).toBe('Waiting for order removal to be indexed...');
		expect(pendingState.explorerLink).toBe('https://explorer.example.com/tx/removeordertxhash');

		expect(switchChain).toHaveBeenCalledWith(mockConfig, { chainId: mockChainId });
		expect(sendTransaction).toHaveBeenCalledWith(mockConfig, {
			to: mockOrderbookAddress,
			data: mockRemoveOrderCalldata
		});
		expect(waitForTransactionReceipt).toHaveBeenCalledWith(mockConfig, { hash: mockTxHash });
		expect(getExplorerLink).toHaveBeenCalledWith(mockTxHash, mockChainId, 'tx');
		expect(awaitSubgraphIndexing).toHaveBeenCalled();

		if (resolveIndexing) {
			resolveIndexing({
				value: {
					txHash: mockTxHash,
					successMessage: 'Order removed successfully'
				}
			});
		} else {
			throw new Error('resolveIndexing is undefined');
		}

		await transactionPromise;

		const finalState = get(transactionStore);
		expect(finalState.status).toBe(TransactionStatus.SUCCESS);
		expect(finalState.message).toBe('Order removed successfully');
		expect(finalState.explorerLink).toBe('https://explorer.example.com/tx/removeordertxhash');
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

	it('should handle subgraph indexing error', async () => {
		(switchChain as Mock).mockResolvedValue({});

		const mockTxHash = '0xremoveordertxhash';
		(sendTransaction as Mock).mockResolvedValue(mockTxHash);
		(waitForTransactionReceipt as Mock).mockResolvedValue({});

		(awaitSubgraphIndexing as Mock).mockResolvedValue({
			error: TIMEOUT_ERROR
		});

		await handleRemoveOrderTransaction({
			config: mockConfig,
			orderbookAddress: mockOrderbookAddress,
			removeOrderCalldata: mockRemoveOrderCalldata,
			chainId: mockChainId,
			subgraphUrl: mockSubgraphUrl
		});

		const state = get(transactionStore);
		expect(state.status).toBe(TransactionStatus.ERROR);
		expect(state.error).toBe(TransactionErrorMessage.TIMEOUT);
	});
});

describe('handleDepositOrWithdrawTransaction', () => {
	const mockConfig = {} as Config;
	const mockChainId = 1;
	const mockSubgraphUrl = 'https://api.thegraph.com/subgraphs/name/test/orderbook';
	const mockVault = {
		token: {
			address: '0xtoken1' as `0x${string}`,
			symbol: 'TKN1'
		},
		orderbook: {
			id: '0xorderbook1' as `0x${string}`
		}
	} as SgVault;
	const mockTransactionCalldata = '0xtransactioncalldata' as unknown as
		| DepositCalldataResult
		| WithdrawCalldataResult;

	const { reset, handleDepositOrWithdrawTransaction } = transactionStore;

	beforeEach(() => {
		vi.resetAllMocks();
		reset();
	});

	afterAll(() => {
		vi.clearAllMocks();
	});

	it('should successfully handle a deposit transaction', async () => {
		(switchChain as Mock).mockResolvedValue({});
		const mockTxHash = '0xdeposittxhash';
		(sendTransaction as Mock).mockResolvedValue(mockTxHash);
		(waitForTransactionReceipt as Mock).mockResolvedValue({});
		(getExplorerLink as Mock).mockResolvedValue('https://explorer.example.com/tx/deposittxhash');

		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		let resolveIndexing3: ((value: unknown) => void) | undefined;
		const indexingPromise3 = new Promise<unknown>((resolve) => {
			resolveIndexing3 = resolve;
		});

		(awaitSubgraphIndexing as Mock).mockReturnValue(indexingPromise3);

		const transactionPromise = handleDepositOrWithdrawTransaction({
			config: mockConfig,
			transactionCalldata: mockTransactionCalldata,
			action: 'deposit',
			chainId: mockChainId,
			vault: mockVault,
			subgraphUrl: mockSubgraphUrl
		});

		await new Promise((resolve) => setTimeout(resolve, 0));

		const pendingState = get(transactionStore);
		expect(pendingState.status).toBe(TransactionStatus.PENDING_SUBGRAPH);
		expect(pendingState.hash).toBe(mockTxHash);
		expect(pendingState.message).toBe('Checking for transaction indexing...');

		if (resolveIndexing3) {
			resolveIndexing3({
				value: {
					txHash: mockTxHash,
					successMessage: 'The deposit was successful.'
				}
			});
		}

		await transactionPromise;

		const finalState = get(transactionStore);
		expect(finalState.status).toBe(TransactionStatus.SUCCESS);
		expect(finalState.message).toBe('The deposit was successful.');
		expect(finalState.explorerLink).toBe('https://explorer.example.com/tx/deposittxhash');
	});

	it('should handle subgraph indexing error', async () => {
		(switchChain as Mock).mockResolvedValue({});

		const mockTxHash = '0xdeposittxhash';
		(sendTransaction as Mock).mockResolvedValue(mockTxHash);

		(waitForTransactionReceipt as Mock).mockResolvedValue({});

		(awaitSubgraphIndexing as Mock).mockResolvedValue({
			error: TIMEOUT_ERROR
		});

		await handleDepositOrWithdrawTransaction({
			config: mockConfig,
			transactionCalldata: mockTransactionCalldata,
			action: 'deposit',
			chainId: mockChainId,
			vault: mockVault,
			subgraphUrl: mockSubgraphUrl
		});

		const state = get(transactionStore);
		expect(state.status).toBe(TransactionStatus.ERROR);
		expect(state.error).toBe(TransactionErrorMessage.TIMEOUT);
	});
});
