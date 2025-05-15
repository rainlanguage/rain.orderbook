import { describe, it, expect, vi, beforeEach } from 'vitest';
import { TransactionStore } from '../lib/models/Transaction';
import { TransactionStatusMessage, TransactionErrorMessage, type TransactionArgs } from '../lib/types/transaction';
import { waitForTransactionReceipt, type Config } from '@wagmi/core';
import { awaitSubgraphIndexing } from '../lib/services/awaitTransactionIndexing';
import { getExplorerLink } from '../lib/services/getExplorerLink';
import { get } from 'svelte/store';
import type { Chain } from 'viem';
import type { ToastLink } from '../lib/types/toast';

vi.mock('@wagmi/core', () => ({
	waitForTransactionReceipt: vi.fn()
}));

vi.mock('../lib/services/awaitTransactionIndexing', () => ({
	awaitSubgraphIndexing: vi.fn(),
	getRemoveOrderConfig: vi.fn(() => ({
		query: 'mock query',
		variables: { txHash: '0x123' }
	}))
}));

vi.mock('../lib/services/getExplorerLink', () => ({
	getExplorerLink: vi.fn()
}));

describe('TransactionStore', () => {
	const mockChain: Chain = {
		id: 1,
		name: 'Ethereum',
		nativeCurrency: {
			name: 'Ether',
			symbol: 'ETH',
			decimals: 18
		},
		rpcUrls: {
			default: { http: ['https://eth.llamarpc.com'] },
			public: { http: ['https://eth.llamarpc.com'] }
		},
		blockExplorers: {
			default: { name: 'Etherscan', url: 'https://etherscan.io' }
		}
	};

	const mockConfig = {
		chains: [mockChain] as const,
		connectors: [],
		storage: {
			getItem: vi.fn(),
			setItem: vi.fn(),
			removeItem: vi.fn(),
			key: 'wagmi'
		},
		state: {
			connections: new Map(),
			chainId: 1,
			status: 'connected',
			current: undefined
		},
		setState: vi.fn(),
		subscribe: vi.fn(),
		destroy: vi.fn(),
		getClient: vi.fn(),
		_internal: {}
	};

	const mockChainId = 1;
	const mockSubgraphUrl = 'https://api.thegraph.com/subgraphs/name/mock';
	const mockTxHash = '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef';
	const mockOrderHash = '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890';
	const mockExplorerLink = 'https://etherscan.io/tx/0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef';
	const mockOnSuccess = vi.fn();
	const mockOnError = vi.fn();

	const mockToastLinks: ToastLink[] = [
		{
			link: mockExplorerLink,
			label: 'View on Explorer'
		}
	];

	let transaction: TransactionStore;

	beforeEach(() => {
		vi.clearAllMocks();
		transaction = new TransactionStore(
			{
				config: mockConfig,
				chainId: mockChainId,
				subgraphUrl: mockSubgraphUrl,
				txHash: mockTxHash,
				orderHash: mockOrderHash,
				errorMessage: 'Transaction failed',
				successMessage: 'Transaction successful',
				queryKey: 'removeOrder',
				toastLinks: mockToastLinks
			} as unknown as TransactionArgs & { config: Config },
			mockOnSuccess,
			mockOnError
		);
		vi.mocked(getExplorerLink).mockResolvedValue(mockExplorerLink);
	});

	it('should initialize with IDLE status', () => {
		const state = get(transaction.state);
		expect(state.status).toBe(TransactionStatusMessage.IDLE);
		expect(state.message).toBe('');
		expect(state.explorerLink).toBe('');
	});

	it('should update state when execute is called', async () => {
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		vi.mocked(waitForTransactionReceipt).mockResolvedValue({} as any);
		vi.mocked(awaitSubgraphIndexing).mockResolvedValue({
			value: {
				txHash: mockTxHash,
				successMessage: 'Order removed successfully'
			}
		});

		await transaction.execute();

		const state = get(transaction.state);
		expect(state.status).toBe(TransactionStatusMessage.SUCCESS);
		expect(state.message).toBe('Order removal indexed successfully.');
		expect(state.explorerLink).toBe(mockExplorerLink);
		expect(mockOnSuccess).toHaveBeenCalled();
	});

	it('should handle transaction receipt failure', async () => {
		vi.mocked(waitForTransactionReceipt).mockRejectedValue(new Error('Transaction failed'));

		await transaction.execute();

		const state = get(transaction.state);
		expect(state.status).toBe(TransactionStatusMessage.ERROR);
		expect(state.message).toBe('Failed to get transaction receipt.');
		expect(mockOnError).toHaveBeenCalled();
	});

	it('should handle subgraph indexing timeout', async () => {
		vi.mocked(waitForTransactionReceipt).mockResolvedValue({} as any);
		vi.mocked(awaitSubgraphIndexing).mockResolvedValue({ error: TransactionErrorMessage.TIMEOUT });

		await transaction.execute();

		const state = get(transaction.state);
		expect(state.status).toBe(TransactionStatusMessage.ERROR);
		expect(state.message).toBe('Subgraph indexing timed out.');
		expect(mockOnError).toHaveBeenCalled();
	});

	it('should handle subgraph indexing failure', async () => {
		vi.mocked(waitForTransactionReceipt).mockResolvedValue({} as any);
		vi.mocked(awaitSubgraphIndexing).mockRejectedValue(new Error('Subgraph error'));

		await transaction.execute();

		const state = get(transaction.state);
		expect(state.status).toBe(TransactionStatusMessage.ERROR);
		expect(state.message).toBe('Failed to index order removal.');
		expect(mockOnError).toHaveBeenCalled();
	});

	it('should handle unknown subgraph indexing error', async () => {
		vi.mocked(waitForTransactionReceipt).mockResolvedValue({} as any);
		vi.mocked(awaitSubgraphIndexing).mockResolvedValue({
			value: {
				txHash: mockTxHash,
				successMessage: 'Order removed successfully'
			}
		});

		await transaction.execute();

		const state = get(transaction.state);
		expect(state.status).toBe(TransactionStatusMessage.SUCCESS);
		expect(state.message).toBe('Order removal indexed successfully.');
		expect(mockOnSuccess).toHaveBeenCalled();
	});

	it('should get current message', () => {
		transaction.state.update((state) => ({
			...state,
			message: 'Test message'
		}));

		expect(transaction.message).toBe('Test message');
	});
}); 