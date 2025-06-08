import { describe, it, expect, vi, beforeEach } from 'vitest';
import { TransactionStore } from '../lib/models/Transaction';
import {
	TransactionStatusMessage,
	TransactionStoreErrorMessage,
	type TransactionArgs,
	TransactionName
} from '../lib/types/transaction';
import { waitForTransactionReceipt, type Config } from '@wagmi/core';
import {
	awaitSubgraphIndexing,
	type AwaitSubgraphConfig
} from '../lib/services/awaitTransactionIndexing';
import { get } from 'svelte/store';
import type { Chain } from 'viem';
import type { ToastLink } from '../lib/types/toast';
import type { SgVault } from '@rainlanguage/orderbook';

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
			status: 'connected',
			current: undefined
		},
		setState: vi.fn(),
		subscribe: vi.fn(),
		destroy: vi.fn(),
		getClient: vi.fn(),
		_internal: {}
	} as unknown as Config;

	const mockVault: SgVault = {
		id: 'vault1',
		vaultId: 'vault1',
		token: {
			id: 'token1',
			address: '0xTokenAddress1',
			name: 'Token1',
			symbol: 'TKN1',
			decimals: '18'
		},
		owner: '0xOwnerAddress',
		ordersAsInput: [],
		ordersAsOutput: [],
		balanceChanges: [],
		balance: '1000000000000000000',
		orderbook: {
			id: '0x00'
		}
	};

	const mockChainId = 1;
	const mockSubgraphUrl = 'https://api.thegraph.com/subgraphs/name/mock';
	const mockTxHash = '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef';
	const mockOrderHash = '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890';
	const mockOnSuccess = vi.fn();
	const mockOnError = vi.fn();

	const mockToastLinks: ToastLink[] = [
		{
			link: 'https://etherscan.io/tx/test-tx',
			label: 'View on Explorer'
		}
	];

	let transaction: TransactionStore;

	const mockAwaitSubgraphConfig: AwaitSubgraphConfig = {
		subgraphUrl: mockSubgraphUrl,
		txHash: mockTxHash,
		successMessage: 'Transaction successful',
		fetchEntityFn: vi.fn(),
		isSuccess: vi.fn()
	};

	beforeEach(() => {
		vi.clearAllMocks();
		transaction = new TransactionStore(
			{
				config: mockConfig,
				chainId: mockChainId,
				subgraphUrl: mockSubgraphUrl,
				txHash: mockTxHash,
				orderHash: mockOrderHash,
				name: TransactionName.REMOVAL,
				errorMessage: 'Transaction failed',
				successMessage: 'Transaction successful',
				queryKey: 'removeOrder',
				toastLinks: mockToastLinks,
				networkKey: 'ethereum',
				awaitSubgraphConfig: mockAwaitSubgraphConfig,
				entity: mockVault
			} as TransactionArgs & { config: Config },
			mockOnSuccess,
			mockOnError
		);
	});

	it('should initialize with IDLE status and correct links', () => {
		const state = get(transaction.state);
		expect(state.status).toBe(TransactionStatusMessage.IDLE);
		expect(state.links).toEqual(mockToastLinks);
	});

	it('should update state when execute is called and keep links', async () => {
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
		expect(state.links).toEqual(mockToastLinks);
		expect(mockOnSuccess).toHaveBeenCalled();
	});

	it('should handle transaction receipt failure', async () => {
		vi.mocked(waitForTransactionReceipt).mockRejectedValue(new Error('Transaction failed'));

		await transaction.execute();

		const state = get(transaction.state);
		expect(state.status).toBe(TransactionStatusMessage.ERROR);
		expect(state.errorDetails).toBe(TransactionStoreErrorMessage.RECEIPT_FAILED);
		expect(state.links).toEqual(mockToastLinks);
		expect(mockOnError).toHaveBeenCalled();
	});

	it('should handle subgraph indexing timeout', async () => {
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		vi.mocked(waitForTransactionReceipt).mockResolvedValue({} as any);
		vi.mocked(awaitSubgraphIndexing).mockResolvedValue({
			error: TransactionStoreErrorMessage.SUBGRAPH_TIMEOUT_ERROR
		});

		await transaction.execute();

		const state = get(transaction.state);
		expect(state.status).toBe(TransactionStatusMessage.ERROR);
		expect(state.errorDetails).toBe(TransactionStoreErrorMessage.SUBGRAPH_TIMEOUT_ERROR);
		expect(state.links).toEqual(mockToastLinks);
		expect(mockOnError).toHaveBeenCalled();
	});

	it('should handle subgraph indexing failure when value is missing', async () => {
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		vi.mocked(waitForTransactionReceipt).mockResolvedValue({} as any);
		vi.mocked(awaitSubgraphIndexing).mockResolvedValue({});

		await transaction.execute();

		const state = get(transaction.state);

		expect(state.status).toBe(TransactionStatusMessage.ERROR);
		expect(state.errorDetails).toBe(TransactionStoreErrorMessage.SUBGRAPH_FAILED);
		expect(state.links).toEqual(mockToastLinks);
		expect(mockOnError).toHaveBeenCalled();
	});

	it('should call onSuccess when execute and indexing are successful', async () => {
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
		expect(state.links).toEqual(mockToastLinks);
		expect(mockOnSuccess).toHaveBeenCalled();
	});
});
