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
		const mockArgs: InternalTransactionArgs = {
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
				isSuccess: (data: SgRemoveOrderWithOrder[]) => data?.length > 0
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

			await manager.createRemoveOrderTransaction(mockArgs);

			expect(TransactionStore).toHaveBeenCalledWith(
				{
					...mockArgs,
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
		});

		it('should execute the transaction after creation', async () => {
			const mockExecute = vi.fn();
			const mockTransaction = { execute: mockExecute };
			vi.mocked(TransactionStore).mockImplementation(
				() => mockTransaction as unknown as TransactionStore
			);

			await manager.createRemoveOrderTransaction(mockArgs);

			expect(mockExecute).toHaveBeenCalled();
		});

		it('should add transaction to store', async () => {
			const mockTransaction = { execute: vi.fn() };
			vi.mocked(TransactionStore).mockImplementation(
				() => mockTransaction as unknown as TransactionStore
			);

			await manager.createRemoveOrderTransaction(mockArgs);

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
		const mockArgs: InternalTransactionArgs = {
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
				isSuccess: (data: SgTransaction) => !!data
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

			await manager.createWithdrawTransaction(mockArgs);

			expect(TransactionStore).toHaveBeenCalledWith(
				{
					...mockArgs,
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
		});

		it('should execute the transaction after creation', async () => {
			const mockExecute = vi.fn();
			const mockTransaction = { execute: mockExecute };
			vi.mocked(TransactionStore).mockImplementation(
				() => mockTransaction as unknown as TransactionStore
			);

			await manager.createWithdrawTransaction(mockArgs);

			expect(mockExecute).toHaveBeenCalled();
		});

		it('should add transaction to store', async () => {
			const mockTransaction = { execute: vi.fn() };
			vi.mocked(TransactionStore).mockImplementation(
				() => mockTransaction as unknown as TransactionStore
			);

			await manager.createWithdrawTransaction(mockArgs);

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

			await manager.createWithdrawTransaction(mockArgs);

			onSuccess!();

			expect(mockAddToast).toHaveBeenCalledWith({
				message: 'Withdrawal successful.',
				type: 'success',
				color: 'green',
				links: expect.any(Array)
			});
			expect(mockQueryClient.invalidateQueries).toHaveBeenCalledWith({
				queryKey: ['0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890']
			});
		});

		it('should handle failed transaction', async () => {
			const mockTransaction = { execute: vi.fn() };
			let onError: () => void;
			vi.mocked(TransactionStore).mockImplementation((args, success, error) => {
				onError = error;
				return mockTransaction as unknown as TransactionStore;
			});

			await manager.createWithdrawTransaction(mockArgs);

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
				subgraphUrl: 'https://api.example.com',
				txHash:
					'0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef' as `0x${string}`,
				chainId: 1,
				networkKey: 'ethereum',
				queryKey: '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
				awaitSubgraphConfig: {
					subgraphUrl: 'https://api.example.com',
					txHash:
						'0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef' as `0x${string}`,
					successMessage: 'Order removed successfully.',
					fetchEntityFn: getTransactionRemoveOrders as typeof getTransactionRemoveOrders,
					isSuccess: (data: SgRemoveOrderWithOrder[]) => data?.length > 0
				}
			});

			onSuccess!();

			expect(mockAddToast).toHaveBeenCalledWith({
				message: 'Order removed successfully.',
				type: 'success',
				color: 'green',
				links: expect.any(Array)
			});
			expect(mockQueryClient.invalidateQueries).toHaveBeenCalledWith({
				queryKey: ['0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890']
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
				subgraphUrl: 'https://api.example.com',
				txHash:
					'0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef' as `0x${string}`,
				chainId: 1,
				networkKey: 'ethereum',
				queryKey: '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
				awaitSubgraphConfig: {
					subgraphUrl: 'https://api.example.com',
					txHash:
						'0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef' as `0x${string}`,
					successMessage: 'Order removed successfully.',
					fetchEntityFn: getTransactionRemoveOrders as typeof getTransactionRemoveOrders,
					isSuccess: (data: SgRemoveOrderWithOrder[]) => data?.length > 0
				}
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
				subgraphUrl: 'https://api.example.com',
				txHash:
					'0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef' as `0x${string}`,
				chainId: 1,
				networkKey: 'ethereum',
				queryKey: '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
				awaitSubgraphConfig: {
					subgraphUrl: 'https://api.example.com',
					txHash:
						'0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef' as `0x${string}`,
					successMessage: 'Order removed successfully.',
					fetchEntityFn: getTransactionRemoveOrders as typeof getTransactionRemoveOrders,
					isSuccess: (data: SgRemoveOrderWithOrder[]) => data?.length > 0
				}
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
