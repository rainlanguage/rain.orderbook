import { describe, it, expect, vi, beforeEach } from 'vitest';
import { TransactionManager } from '../lib/providers/transactions/TransactionManager';
import { TransactionStore } from '../lib/models/Transaction';
import type { QueryClient } from '@tanstack/svelte-query';
import type { Config } from '@wagmi/core';
import type { ToastProps } from '../lib/types/toast';
import { TransactionName, type InternalTransactionArgs } from '../lib/types/transaction';
import { getExplorerLink } from '../lib/services/getExplorerLink';
import {
	RaindexClient,
	RaindexOrder,
	RaindexTransaction,
	RaindexVault,
	type Address
} from '@rainlanguage/orderbook';
import type { AwaitSubgraphConfig } from '$lib/services/awaitTransactionIndexing';

vi.mock('../lib/models/Transaction', () => ({
	TransactionStore: vi.fn()
}));

vi.mock('../lib/services/getExplorerLink', () => ({
	getExplorerLink: vi.fn()
}));

vi.mock('@rainlanguage/orderbook', () => ({
	getTransactionRemoveOrders: vi.fn(),
	getTransaction: vi.fn(),
	getTransactionAddOrders: vi.fn()
}));

describe('TransactionManager', () => {
	let mockQueryClient: QueryClient;
	let mockAddToast: (toast: Omit<ToastProps, 'id'>) => void;
	let mockWagmiConfig: Config;
	let manager: TransactionManager;

	const mockRaindexClient = {
		getRemoveOrdersForTransaction: vi.fn(),
		getTransaction: vi.fn(),
		getAddOrdersForTransaction: vi.fn()
	} as unknown as RaindexClient;

	const mockSgOrderEntity = {
		id: 'mockOrderEntityId',
		orderbook: 'mockOrderbook',
		getRemoveOrdersForTransaction: vi.fn()
	} as unknown as RaindexOrder;

	const mockSgVaultEntity = {
		token: { symbol: 'MOCKVAULT', decimals: '18' },
		vaultId: 'mockVaultEntityId',
		id: 'mockVaultEntityId'
	} as unknown as RaindexVault;

	const mockBaseArgs: InternalTransactionArgs = {
		txHash: '0xcallbacktxhash' as `0x${string}`,
		chainId: 1,
		queryKey: '0xcallbackkey'
	};

	beforeEach(() => {
		vi.clearAllMocks();

		mockQueryClient = {
			invalidateQueries: vi.fn()
		} as unknown as QueryClient;

		mockAddToast = vi.fn();

		mockWagmiConfig = {} as Config;
		manager = new TransactionManager(mockQueryClient, mockAddToast, mockWagmiConfig);
	});

	describe('initialization', () => {
		it('should initialize with empty transactions store', () => {
			const transactions = manager.getTransactions();
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			let storeValue: any[] = [];
			transactions.subscribe((value) => {
				storeValue = value;
			});
			expect(storeValue).toEqual([]);
		});
	});

	describe('createRemoveOrderTransaction', () => {
		const removeOrderMockArgs: InternalTransactionArgs & {
			raindexClient: RaindexClient;
			entity: RaindexOrder;
		} = {
			txHash: '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef' as `0x${string}`,
			chainId: 1,
			queryKey: '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
			entity: mockSgOrderEntity,
			raindexClient: mockRaindexClient
		};

		const fullMockArgsForExpectation: InternalTransactionArgs & {
			awaitSubgraphConfig: AwaitSubgraphConfig;
		} = {
			...removeOrderMockArgs,
			awaitSubgraphConfig: {
				chainId: removeOrderMockArgs.chainId,
				orderbook: removeOrderMockArgs.entity.orderbook,
				txHash: removeOrderMockArgs.txHash,
				successMessage: 'Order removed successfully.',
				fetchEntityFn: expect.any(Function),
				isSuccess: expect.any(Function)
			}
		};

		beforeEach(() => {
			vi.mocked(getExplorerLink).mockResolvedValue(
				'https://explorer.example.com/tx/0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef'
			);
		});

		it('should create a transaction with correct parameters', async () => {
			const mockTransaction = { execute: vi.fn() };
			vi.mocked(TransactionStore).mockImplementation(
				() => mockTransaction as unknown as TransactionStore
			);

			await manager.createRemoveOrderTransaction(removeOrderMockArgs);

			expect(TransactionStore).toHaveBeenCalledWith(
				expect.objectContaining({
					...fullMockArgsForExpectation,
					name: TransactionName.REMOVAL,
					errorMessage: 'Order removal failed.',
					successMessage: 'Order removed successfully.',
					queryKey: removeOrderMockArgs.queryKey,
					toastLinks: [
						{
							link: 'https://explorer.example.com/tx/0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
							label: 'View on explorer'
						}
					],
					config: mockWagmiConfig,
					awaitSubgraphConfig: expect.objectContaining({
						chainId: removeOrderMockArgs.chainId,
						orderbook: removeOrderMockArgs.entity.orderbook,
						txHash: removeOrderMockArgs.txHash,
						successMessage: 'Order removed successfully.',
						fetchEntityFn: expect.any(Function),
						isSuccess: expect.any(Function)
					})
				}),
				expect.any(Function),
				expect.any(Function)
			);

			const removeOrderCallArgs = vi.mocked(TransactionStore).mock.calls[0][0];
			const removeOrderIsSuccessFn = removeOrderCallArgs.awaitSubgraphConfig!.isSuccess;

			expect(
				removeOrderIsSuccessFn([
					{
						id: 'order1',
						transaction: {
							id: 'tx1',
							from: '0xfrom',
							blockNumber: '123',
							timestamp: '1678886400'
						}
					}
				] as unknown as RaindexOrder[])
			).toBe(true);
			expect(removeOrderIsSuccessFn([] as RaindexOrder[])).toBe(false);
			expect(
				removeOrderIsSuccessFn({
					id: 'tx1',
					from: '0xfrom',
					blockNumber: '123',
					timestamp: '1678886400'
				} as unknown as RaindexTransaction)
			).toBe(false);
		});

		it('should execute the transaction after creation', async () => {
			const mockExecute = vi.fn();
			const mockTransaction = { execute: mockExecute };
			vi.mocked(TransactionStore).mockImplementation(
				() => mockTransaction as unknown as TransactionStore
			);

			await manager.createRemoveOrderTransaction(removeOrderMockArgs);

			expect(mockExecute).toHaveBeenCalled();
		});

		it('should add transaction to store', async () => {
			const mockTransaction = { execute: vi.fn() };
			vi.mocked(TransactionStore).mockImplementation(
				() => mockTransaction as unknown as TransactionStore
			);

			await manager.createRemoveOrderTransaction(removeOrderMockArgs);

			const transactions = manager.getTransactions();
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			let storeValue: any[] = [];
			transactions.subscribe((value) => {
				storeValue = value;
			});
			expect(storeValue).toContain(mockTransaction);
		});
	});

	describe('createWithdrawTransaction', () => {
		const withdrawMockArgs: InternalTransactionArgs & {
			raindexClient: RaindexClient;
			entity: RaindexVault;
		} = {
			txHash: '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef' as `0x${string}`,
			chainId: 1,
			queryKey: '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
			entity: mockSgVaultEntity,
			raindexClient: mockRaindexClient
		};

		const fullMockArgsForExpectation: InternalTransactionArgs & {
			awaitSubgraphConfig: AwaitSubgraphConfig;
		} = {
			...withdrawMockArgs, // Spreads withdrawMockArgs including entity
			awaitSubgraphConfig: {
				chainId: withdrawMockArgs.chainId,
				orderbook: withdrawMockArgs.entity.orderbook,
				txHash: withdrawMockArgs.txHash,
				successMessage: 'Withdrawal successful.',
				fetchEntityFn: expect.any(Function),
				isSuccess: expect.any(Function)
			}
		};

		beforeEach(() => {
			vi.mocked(getExplorerLink).mockResolvedValue(
				'https://explorer.example.com/tx/0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef'
			);
		});

		it('should create a transaction with correct parameters', async () => {
			const mockTransaction = { execute: vi.fn() };
			vi.mocked(TransactionStore).mockImplementation(
				() => mockTransaction as unknown as TransactionStore
			);

			await manager.createWithdrawTransaction(withdrawMockArgs);

			expect(TransactionStore).toHaveBeenCalledWith(
				expect.objectContaining({
					...fullMockArgsForExpectation,
					name: TransactionName.WITHDRAWAL,
					errorMessage: 'Withdrawal failed.',
					successMessage: 'Withdrawal successful.',
					queryKey: withdrawMockArgs.queryKey,
					toastLinks: [
						{
							link: `/vaults/${withdrawMockArgs.chainId}-${withdrawMockArgs.entity.orderbook}-${withdrawMockArgs.queryKey}`,
							label: 'View vault'
						},
						{
							link: 'https://explorer.example.com/tx/0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
							label: 'View on explorer'
						}
					],
					config: mockWagmiConfig,
					awaitSubgraphConfig: expect.objectContaining({
						chainId: withdrawMockArgs.chainId,
						orderbook: withdrawMockArgs.entity.orderbook,
						txHash: withdrawMockArgs.txHash,
						successMessage: 'Withdrawal successful.',
						fetchEntityFn: expect.any(Function),
						isSuccess: expect.any(Function)
					})
				}),
				expect.any(Function),
				expect.any(Function)
			);

			const withdrawCallArgs = vi.mocked(TransactionStore).mock.calls[0][0];
			const withdrawIsSuccessFn = withdrawCallArgs.awaitSubgraphConfig!.isSuccess;

			expect(withdrawIsSuccessFn({ id: '0x0123' } as unknown as RaindexTransaction)).toBe(true);
			expect(withdrawIsSuccessFn(null as unknown as RaindexTransaction)).toBe(false);
			expect(withdrawIsSuccessFn(undefined as unknown as RaindexTransaction)).toBe(false);
			expect(withdrawIsSuccessFn('' as unknown as RaindexTransaction)).toBe(false);
			expect(withdrawIsSuccessFn(0 as unknown as RaindexTransaction)).toBe(false);
		});

		it('should execute the transaction after creation', async () => {
			const mockExecute = vi.fn();
			const mockTransaction = { execute: mockExecute };
			vi.mocked(TransactionStore).mockImplementation(
				() => mockTransaction as unknown as TransactionStore
			);

			await manager.createWithdrawTransaction(withdrawMockArgs);

			expect(mockExecute).toHaveBeenCalled();
		});

		it('should add transaction to store', async () => {
			const mockTransaction = { execute: vi.fn() };
			vi.mocked(TransactionStore).mockImplementation(
				() => mockTransaction as unknown as TransactionStore
			);

			await manager.createWithdrawTransaction(withdrawMockArgs);

			const transactions = manager.getTransactions();
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			let storeValue: any[] = [];
			transactions.subscribe((value) => {
				storeValue = value;
			});
			expect(storeValue).toContain(mockTransaction);
		});

		it('should handle successful transaction', async () => {
			const mockTransaction = { execute: vi.fn() };
			let onSuccess: () => void;
			vi.mocked(TransactionStore).mockImplementation((args, success) => {
				onSuccess = success;
				return mockTransaction as unknown as TransactionStore;
			});

			const testSpecificArgs: InternalTransactionArgs & {
				raindexClient: RaindexClient;
				entity: RaindexVault;
			} = {
				...withdrawMockArgs, // Use base withdraw args
				queryKey: '0xvaultid' // Override queryKey for this specific test
			};

			await manager.createWithdrawTransaction(testSpecificArgs);

			onSuccess!();

			expect(mockQueryClient.invalidateQueries).toHaveBeenCalledWith({
				queryKey: ['0xvaultid']
			});
		});

		it('should handle failed transaction', async () => {
			const mockTransaction = { execute: vi.fn() };
			let onError: () => void;
			vi.mocked(TransactionStore).mockImplementation((args, success, error) => {
				onError = error;
				return mockTransaction as unknown as TransactionStore;
			});

			await manager.createWithdrawTransaction(withdrawMockArgs);

			onError!();

			expect(mockAddToast).toHaveBeenCalledWith({
				message: 'Withdrawal failed.',
				type: 'error',
				color: 'red',
				links: expect.any(Array)
			});
		});
	});

	describe('transaction callbacks', () => {
		it('should handle successful transaction', async () => {
			const mockTransaction = { execute: vi.fn() };
			let onSuccess: () => void;
			vi.mocked(TransactionStore).mockImplementation((args, success) => {
				onSuccess = success;
				return mockTransaction as unknown as TransactionStore;
			});

			await manager.createRemoveOrderTransaction({
				...mockBaseArgs,
				queryKey: '0xsuccesskey',
				entity: mockSgOrderEntity,
				raindexClient: mockRaindexClient
			});

			onSuccess!();

			expect(mockAddToast).not.toHaveBeenCalled();
			expect(mockQueryClient.invalidateQueries).toHaveBeenCalledWith({
				queryKey: ['0xsuccesskey']
			});
		});

		it('should handle failed transaction', async () => {
			const mockTransaction = { execute: vi.fn() };
			let onError: () => void;
			vi.mocked(TransactionStore).mockImplementation((args, success, error) => {
				onError = error;
				return mockTransaction as unknown as TransactionStore;
			});

			await manager.createRemoveOrderTransaction({
				...mockBaseArgs,
				queryKey: '0xfailkey',
				entity: mockSgOrderEntity,
				raindexClient: mockRaindexClient
			});

			onError!();

			expect(mockAddToast).toHaveBeenCalledWith({
				message: 'Order removal failed.',
				type: 'error',
				color: 'red',
				links: expect.any(Array)
			});
		});
	});

	describe('clearTransactions', () => {
		it('should clear all transactions from store', async () => {
			const mockTransaction = { execute: vi.fn() };
			vi.mocked(TransactionStore).mockImplementation(
				() => mockTransaction as unknown as TransactionStore
			);

			await manager.createRemoveOrderTransaction({
				...mockBaseArgs,
				queryKey: '0xclearkey',
				entity: mockSgOrderEntity,
				raindexClient: mockRaindexClient
			});

			manager.clearTransactions();

			const transactions = manager.getTransactions();
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			let storeValue: any[] = [];
			transactions.subscribe((value) => {
				storeValue = value;
			});
			expect(storeValue).toEqual([]);
		});
	});

	describe('createApprovalTransaction', () => {
		const mockEntity = {
			token: {
				symbol: 'TEST',
				decimals: '18'
			}
		} as unknown as RaindexVault;
		const mockArgs: InternalTransactionArgs = {
			txHash: '0xapprovehash' as `0x${string}`,
			chainId: 1,
			queryKey: '0xvaultid'
		};

		beforeEach(() => {
			vi.mocked(getExplorerLink).mockResolvedValue('https://explorer.example.com/tx/0xapprovehash');
		});

		it('should create a transaction with correct parameters when entity is provided', async () => {
			const mockTransaction = { execute: vi.fn() };
			vi.mocked(TransactionStore).mockImplementation(
				() => mockTransaction as unknown as TransactionStore
			);

			await manager.createApprovalTransaction({ ...mockArgs, entity: mockEntity });

			expect(TransactionStore).toHaveBeenCalledWith(
				{
					...mockArgs,
					entity: mockEntity,
					name: 'Approving TEST spend',
					errorMessage: 'Approval failed.',
					successMessage: 'Approval successful.',
					queryKey: mockArgs.queryKey,
					toastLinks: [
						{
							link: `/vaults/${mockArgs.chainId}-${mockEntity.orderbook}-${mockArgs.queryKey}`,
							label: 'View vault'
						},
						{
							link: 'https://explorer.example.com/tx/0xapprovehash',
							label: 'View on explorer'
						}
					],
					config: mockWagmiConfig
				},
				expect.any(Function),
				expect.any(Function)
			);
		});

		it('should create a transaction with correct parameters when entity is not provided', async () => {
			const mockTransaction = { execute: vi.fn() };
			vi.mocked(TransactionStore).mockImplementation(
				() => mockTransaction as unknown as TransactionStore
			);

			// entity is deliberately omitted here
			await manager.createApprovalTransaction({ ...mockArgs });

			expect(TransactionStore).toHaveBeenCalledWith(
				{
					...mockArgs,
					name: 'Approving token spend',
					errorMessage: 'Approval failed.',
					successMessage: 'Approval successful.',
					queryKey: mockArgs.queryKey,
					toastLinks: [
						{
							link: 'https://explorer.example.com/tx/0xapprovehash',
							label: 'View on explorer'
						}
					],
					config: mockWagmiConfig
				},
				expect.any(Function),
				expect.any(Function)
			);
		});
	});

	describe('createDepositTransaction', () => {
		const mockEntity = {
			token: {
				symbol: 'TEST',
				decimals: '18'
			}
		} as unknown as RaindexVault;
		const mockArgs: InternalTransactionArgs & {
			amount: string;
			entity: RaindexVault;
			raindexClient: RaindexClient;
		} = {
			txHash: '0xdeposithash' as `0x${string}`,
			chainId: 1,
			queryKey: '0xvaultid',
			entity: mockEntity,
			amount: '1000',
			raindexClient: mockRaindexClient
		};

		beforeEach(() => {
			vi.mocked(getExplorerLink).mockResolvedValue('https://explorer.example.com/tx/0xdeposithash');
		});

		it('should create a transaction with correct parameters including formatted amount', async () => {
			const mockTransaction = { execute: vi.fn() };
			vi.mocked(TransactionStore).mockImplementation(
				() => mockTransaction as unknown as TransactionStore
			);

			await manager.createDepositTransaction(mockArgs);

			expect(TransactionStore).toHaveBeenCalledWith(
				expect.objectContaining({
					...mockArgs,
					name: `Depositing ${mockArgs.amount} ${mockEntity.token.symbol}`,
					errorMessage: 'Deposit failed.',
					successMessage: 'Deposit successful.',
					queryKey: mockArgs.queryKey,
					toastLinks: [
						{
							link: `/vaults/${mockArgs.chainId}-${mockEntity.orderbook}-${mockArgs.queryKey}`,
							label: 'View vault'
						},
						{
							link: 'https://explorer.example.com/tx/0xdeposithash',
							label: 'View on explorer'
						}
					],
					config: mockWagmiConfig,
					awaitSubgraphConfig: expect.objectContaining({
						chainId: mockArgs.chainId,
						orderbook: mockEntity.orderbook,
						txHash: mockArgs.txHash,
						successMessage: 'Deposit successful.',
						fetchEntityFn: expect.any(Function),
						isSuccess: expect.any(Function)
					})
				}),
				expect.any(Function),
				expect.any(Function)
			);
		});

		it('should execute the transaction after creation', async () => {
			const mockExecute = vi.fn();
			const mockTransaction = { execute: mockExecute };
			vi.mocked(TransactionStore).mockImplementation(
				() => mockTransaction as unknown as TransactionStore
			);

			await manager.createDepositTransaction(mockArgs);

			expect(mockExecute).toHaveBeenCalled();
		});

		it('should add transaction to store', async () => {
			const mockTransaction = { execute: vi.fn() };
			vi.mocked(TransactionStore).mockImplementation(
				() => mockTransaction as unknown as TransactionStore
			);

			await manager.createDepositTransaction(mockArgs);

			const transactions = manager.getTransactions();
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			let storeValue: any[] = [];
			transactions.subscribe((value) => {
				storeValue = value;
			});
			expect(storeValue).toContain(mockTransaction);
		});

		it('should handle successful transaction', async () => {
			const mockTransaction = { execute: vi.fn() };
			let onSuccess: () => void;
			vi.mocked(TransactionStore).mockImplementation((args, success) => {
				onSuccess = success;
				return mockTransaction as unknown as TransactionStore;
			});

			await manager.createDepositTransaction(mockArgs);

			onSuccess!();

			expect(mockQueryClient.invalidateQueries).toHaveBeenCalledWith({
				queryKey: ['0xvaultid']
			});
		});

		it('should handle failed transaction', async () => {
			const mockTransaction = { execute: vi.fn() };
			let onError: () => void;
			vi.mocked(TransactionStore).mockImplementation((args, success, error) => {
				onError = error;
				return mockTransaction as unknown as TransactionStore;
			});

			await manager.createDepositTransaction(mockArgs);

			onError!();

			expect(mockAddToast).toHaveBeenCalledWith({
				message: 'Deposit failed.',
				type: 'error',
				color: 'red',
				links: expect.any(Array)
			});
		});
	});

	describe('createAddOrderTransaction', () => {
		const addOrderMockArgs: InternalTransactionArgs & {
			raindexClient: RaindexClient;
			orderbook: Address;
		} = {
			txHash: '0xaddordertxhash' as `0x${string}`,
			chainId: 1,
			queryKey: 'myNewStrategyDeployment',
			raindexClient: mockRaindexClient,
			orderbook: '0xorderbook' as Address
		};

		const fullMockArgsForExpectation: InternalTransactionArgs & {
			awaitSubgraphConfig: AwaitSubgraphConfig;
		} = {
			...addOrderMockArgs,
			awaitSubgraphConfig: {
				chainId: addOrderMockArgs.chainId,
				orderbook: addOrderMockArgs.orderbook,
				txHash: addOrderMockArgs.txHash,
				successMessage: 'Strategy deployed successfully.',
				fetchEntityFn: expect.any(Function),
				isSuccess: expect.any(Function)
			}
		};

		beforeEach(() => {
			vi.mocked(getExplorerLink).mockResolvedValue(
				'https://explorer.example.com/tx/0xaddordertxhash'
			);
		});

		it('should create a transaction with correct parameters', async () => {
			const mockTransaction = { execute: vi.fn() };
			vi.mocked(TransactionStore).mockImplementation(
				() => mockTransaction as unknown as TransactionStore
			);

			await manager.createAddOrderTransaction(addOrderMockArgs);

			expect(TransactionStore).toHaveBeenCalledWith(
				expect.objectContaining({
					...fullMockArgsForExpectation,
					name: 'Deploying order',
					errorMessage: 'Deployment failed.',
					successMessage: 'Order deployed successfully.',
					queryKey: addOrderMockArgs.queryKey,
					toastLinks: [
						{
							link: 'https://explorer.example.com/tx/0xaddordertxhash',
							label: 'View on explorer'
						}
					],
					config: mockWagmiConfig,
					awaitSubgraphConfig: expect.objectContaining({
						chainId: addOrderMockArgs.chainId,
						orderbook: addOrderMockArgs.orderbook,
						txHash: addOrderMockArgs.txHash,
						successMessage: 'Order deployed successfully.',
						fetchEntityFn: expect.any(Function),
						isSuccess: expect.any(Function)
					})
				}),
				expect.any(Function), // onSuccess
				expect.any(Function) // onError
			);

			const addOrderCallArgs = vi.mocked(TransactionStore).mock.calls[0][0];
			const addOrderIsSuccessFn = addOrderCallArgs.awaitSubgraphConfig!.isSuccess;
			const addOrderFetchEntityFn = addOrderCallArgs.awaitSubgraphConfig!.fetchEntityFn;

			expect(addOrderFetchEntityFn.name).toBe('bound spy');
			expect(
				addOrderIsSuccessFn([
					{
						id: 'order1',
						orderHash: '0xneworderhash',
						transaction: { id: 'tx1' }
					}
				] as unknown as RaindexOrder[])
			).toBe(true);
			expect(addOrderIsSuccessFn([] as RaindexOrder[])).toBe(false);
		});

		it('should execute the transaction after creation', async () => {
			const mockExecute = vi.fn();
			const mockTransaction = { execute: mockExecute };
			vi.mocked(TransactionStore).mockImplementation(
				() => mockTransaction as unknown as TransactionStore
			);

			await manager.createAddOrderTransaction(addOrderMockArgs);

			expect(mockExecute).toHaveBeenCalled();
		});

		it('should add transaction to store', async () => {
			const mockTransaction = { execute: vi.fn() };
			vi.mocked(TransactionStore).mockImplementation(
				() => mockTransaction as unknown as TransactionStore
			);

			await manager.createAddOrderTransaction(addOrderMockArgs);

			const transactions = manager.getTransactions();
			let storeValue: unknown[] = [];
			transactions.subscribe((value) => {
				storeValue = value;
			});
			expect(storeValue).toContain(mockTransaction);
		});

		it('should handle successful transaction', async () => {
			const mockTransaction = { execute: vi.fn() };
			let onSuccess: (newOrderHash?: string) => void;
			vi.mocked(TransactionStore).mockImplementation((args, success) => {
				onSuccess = success;
				return mockTransaction as unknown as TransactionStore;
			});
			const newOrderHash = '0xneworderhashfromcallback';

			await manager.createAddOrderTransaction(addOrderMockArgs);

			onSuccess!(newOrderHash); // Simulate successful execution and subgraph indexing providing a new order hash

			expect(mockQueryClient.invalidateQueries).toHaveBeenCalledWith({
				queryKey: [addOrderMockArgs.queryKey]
			});
		});

		it('should handle failed transaction', async () => {
			const mockTransaction = { execute: vi.fn() };
			let onError: () => void;
			vi.mocked(TransactionStore).mockImplementation((args, success, error) => {
				onError = error;
				return mockTransaction as unknown as TransactionStore;
			});

			await manager.createAddOrderTransaction(addOrderMockArgs);

			onError!();

			expect(mockAddToast).toHaveBeenCalledWith({
				message: 'Deployment failed.',
				type: 'error',
				color: 'red',
				links: [
					{
						link: 'https://explorer.example.com/tx/0xaddordertxhash',
						label: 'View on explorer'
					}
				]
			});
		});
	});
});
