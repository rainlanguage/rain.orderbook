import { describe, it, expect, vi, beforeEach } from 'vitest';
import { TransactionManager } from '../lib/providers/transactions/TransactionManager';
import { TransactionStore } from '../lib/models/Transaction';
import type { QueryClient } from '@tanstack/svelte-query';
import type { Config } from '@wagmi/core';
import type { ToastProps } from '../lib/types/toast';
import { TransactionName, type InternalTransactionArgs } from '../lib/types/transaction';
import { getExplorerLink } from '../lib/services/getExplorerLink';

vi.mock('../lib/models/Transaction', () => ({
	TransactionStore: vi.fn()
}));

vi.mock('../lib/services/getExplorerLink', () => ({
	getExplorerLink: vi.fn()
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
			orderHash:
				'0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890' as `0x${string}`,
			chainId: 1,
			networkKey: 'ethereum'
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
					queryKey: mockArgs.orderHash,
					toastLinks: [
						{
							link: `/orders/${mockArgs.networkKey}-${mockArgs.orderHash}`,
							label: 'View Order'
						},
						{
							link: 'https://explorer.example.com/tx/0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
							label: 'View transaction'
						}
					],
					config: mockWagmiConfig
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
				orderHash:
					'0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890' as `0x${string}`,
				chainId: 1,
				networkKey: 'ethereum'
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
				orderHash:
					'0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890' as `0x${string}`,
				chainId: 1,
				networkKey: 'ethereum'
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
				orderHash:
					'0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890' as `0x${string}`,
				chainId: 1,
				networkKey: 'ethereum'
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
