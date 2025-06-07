import { describe, it, expect, vi, beforeEach } from 'vitest';
import { TransactionManager } from '../lib/providers/transactions/TransactionManager';
import { TransactionStore } from '../lib/models/Transaction';
import type { QueryClient } from '@tanstack/svelte-query';
import type { Config } from '@wagmi/core';
import type { ToastProps } from '../lib/types/toast';
import { TransactionName, type InternalTransactionArgs } from '../lib/types/transaction';
import { getExplorerLink } from '../lib/services/getExplorerLink';
import {
	getTransaction,
	getTransactionRemoveOrders,
	type SgRemoveOrderWithOrder,
	type SgTransaction
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
	getTransaction: vi.fn()
}));

describe('TransactionManager', () => {
	let mockQueryClient: QueryClient;
	let mockAddToast: (toast: Omit<ToastProps, 'id'>) => void;
	let mockWagmiConfig: Config;
	let manager: TransactionManager;

	// Define a base mock args for callback tests to use
	const mockBaseArgs: InternalTransactionArgs = {
		subgraphUrl: 'https://api.example.com',
		txHash: '0xcallbacktxhash' as `0x${string}`,
		chainId: 1,
		networkKey: 'ethereum',
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
		const mockArgs: Omit<InternalTransactionArgs, 'awaitSubgraphConfig'> = {
			subgraphUrl: 'https://api.example.com',
			txHash: '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef' as `0x${string}`,
			chainId: 1,
			networkKey: 'ethereum',
			queryKey: '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890'
		};

		const fullMockArgsForExpectation: InternalTransactionArgs & {
			awaitSubgraphConfig: AwaitSubgraphConfig;
		} = {
			...mockArgs,
			subgraphUrl: 'https://api.example.com',
			txHash: '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef' as `0x${string}`,
			chainId: 1,
			networkKey: 'ethereum',
			queryKey: '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
			awaitSubgraphConfig: {
				subgraphUrl: 'https://api.example.com',
				txHash:
					'0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef' as `0x${string}`,
				successMessage: 'Order removed successfully.',
				fetchEntityFn: getTransactionRemoveOrders as typeof getTransactionRemoveOrders,
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

			await manager.createRemoveOrderTransaction(mockArgs as InternalTransactionArgs);

			expect(TransactionStore).toHaveBeenCalledWith(
				{
					...fullMockArgsForExpectation,
					name: TransactionName.REMOVAL,
					errorMessage: 'Order removal failed.',
					successMessage: 'Order removed successfully.',
					queryKey: mockArgs.queryKey,
					toastLinks: [
						{
							link: `/orders/${mockArgs.networkKey}-${mockArgs.queryKey}`,
							label: 'View Order'
						},
						{
							link: 'https://explorer.example.com/tx/0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
							label: 'View transaction'
						}
					],
					config: mockWagmiConfig,
					awaitSubgraphConfig: {
						subgraphUrl: mockArgs.subgraphUrl,
						txHash: mockArgs.txHash,
						successMessage: 'Order removed successfully.',
						fetchEntityFn: getTransactionRemoveOrders as typeof getTransactionRemoveOrders,
						isSuccess: expect.any(Function)
					}
				},
				expect.any(Function),
				expect.any(Function)
			);

			const removeOrderCallArgs = vi.mocked(TransactionStore).mock.calls[0][0];
			const removeOrderIsSuccessFn = removeOrderCallArgs.awaitSubgraphConfig.isSuccess;
			expect(
				removeOrderIsSuccessFn([
					{
						// SgRemoveOrderWithOrder structure:
						// NO 'id' field at this top level for SgRemoveOrderWithOrder itself.
						transaction: {
							id: 'tx1',
							from: '0xfrom',
							blockNumber: '123',
							timestamp: '1678886400'
						},
						order: { id: 'order1' } // SgOrder needs at least an id
					}
				] as SgRemoveOrderWithOrder[])
			).toBe(true);
			expect(removeOrderIsSuccessFn([] as SgRemoveOrderWithOrder[])).toBe(false);
			// Test with a non-array SgTransaction to ensure it's handled as false by the original logic
			expect(
				removeOrderIsSuccessFn({
					id: 'tx1',
					from: '0xfrom',
					blockNumber: '123',
					timestamp: '1678886400'
				} as SgTransaction)
			).toBe(false);
		});

		it('should execute the transaction after creation', async () => {
			const mockExecute = vi.fn();
			const mockTransaction = { execute: mockExecute };
			vi.mocked(TransactionStore).mockImplementation(
				() => mockTransaction as unknown as TransactionStore
			);

			await manager.createRemoveOrderTransaction(mockArgs as InternalTransactionArgs);

			expect(mockExecute).toHaveBeenCalled();
		});

		it('should add transaction to store', async () => {
			const mockTransaction = { execute: vi.fn() };
			vi.mocked(TransactionStore).mockImplementation(
				() => mockTransaction as unknown as TransactionStore
			);

			await manager.createRemoveOrderTransaction(mockArgs as InternalTransactionArgs);

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
		const mockArgs: Omit<InternalTransactionArgs, 'awaitSubgraphConfig'> = {
			subgraphUrl: 'https://api.example.com',
			txHash: '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef' as `0x${string}`,
			chainId: 1,
			networkKey: 'ethereum',
			queryKey: '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890'
		};

		const fullMockArgsForExpectation: InternalTransactionArgs & {
			awaitSubgraphConfig: AwaitSubgraphConfig;
		} = {
			...mockArgs,
			subgraphUrl: 'https://api.example.com',
			txHash: '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef' as `0x${string}`,
			chainId: 1,
			networkKey: 'ethereum',
			queryKey: '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
			awaitSubgraphConfig: {
				subgraphUrl: 'https://api.example.com',
				txHash:
					'0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef' as `0x${string}`,
				successMessage: 'Withdrawal successful.',
				fetchEntityFn: getTransaction as typeof getTransaction,
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

			await manager.createWithdrawTransaction(mockArgs as InternalTransactionArgs);

			expect(TransactionStore).toHaveBeenCalledWith(
				{
					...fullMockArgsForExpectation,
					name: TransactionName.WITHDRAWAL,
					errorMessage: 'Withdrawal failed.',
					successMessage: 'Withdrawal successful.',
					queryKey: mockArgs.queryKey,
					toastLinks: [
						{
							link: `/vaults/${mockArgs.networkKey}-${mockArgs.queryKey}`,
							label: 'View vault'
						},
						{
							link: 'https://explorer.example.com/tx/0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
							label: 'View transaction'
						}
					],
					config: mockWagmiConfig,
					awaitSubgraphConfig: {
						subgraphUrl: mockArgs.subgraphUrl,
						txHash: mockArgs.txHash,
						successMessage: 'Withdrawal successful.',
						fetchEntityFn: getTransaction as typeof getTransaction,
						isSuccess: expect.any(Function)
					}
				},
				expect.any(Function),
				expect.any(Function)
			);

			const withdrawCallArgs = vi.mocked(TransactionStore).mock.calls[0][0];
			const withdrawIsSuccessFn = withdrawCallArgs.awaitSubgraphConfig.isSuccess;
			expect(withdrawIsSuccessFn({ id: 'tx1' } as SgTransaction)).toBe(true);
			expect(withdrawIsSuccessFn(null as unknown as SgTransaction)).toBe(false);
			expect(withdrawIsSuccessFn(undefined as unknown as SgTransaction)).toBe(false);
			expect(withdrawIsSuccessFn('' as unknown as SgTransaction)).toBe(false);
			expect(withdrawIsSuccessFn(0 as unknown as SgTransaction)).toBe(false);
		});

		it('should execute the transaction after creation', async () => {
			const mockExecute = vi.fn();
			const mockTransaction = { execute: mockExecute };
			vi.mocked(TransactionStore).mockImplementation(
				() => mockTransaction as unknown as TransactionStore
			);

			await manager.createWithdrawTransaction(mockArgs as InternalTransactionArgs);

			expect(mockExecute).toHaveBeenCalled();
		});

		it('should add transaction to store', async () => {
			const mockTransaction = { execute: vi.fn() };
			vi.mocked(TransactionStore).mockImplementation(
				() => mockTransaction as unknown as TransactionStore
			);

			await manager.createWithdrawTransaction(mockArgs as InternalTransactionArgs);

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

			const depositMockArgs = {
				...mockArgs,
				queryKey: '0xvaultid'
			};

			await manager.createWithdrawTransaction(depositMockArgs as InternalTransactionArgs);

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

			await manager.createWithdrawTransaction(mockArgs as InternalTransactionArgs);

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
				queryKey: '0xsuccesskey' // Override for specific test
			});

			onSuccess!();

			expect(mockAddToast).not.toHaveBeenCalled();
			expect(mockQueryClient.invalidateQueries).toHaveBeenCalledWith({
				queryKey: ['0xsuccesskey'] // ensure this matches the queryKey used in this specific test call
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
				queryKey: '0xfailkey' // Override for specific test
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
				queryKey: '0xclearkey' // Override for specific test
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
});
