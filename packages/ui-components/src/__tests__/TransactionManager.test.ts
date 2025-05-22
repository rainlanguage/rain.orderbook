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
	type SgTransaction,
	type SgVault,
	type SgOrder
} from '@rainlanguage/orderbook';
import { formatUnits } from 'viem';
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

	const mockSgOrderEntity = {
		id: 'mockOrderEntityId'
	} as unknown as SgOrder;

	const mockSgVaultEntity = {
		token: { symbol: 'MOCKVAULT', decimals: '18' },
		vaultId: 'mockVaultEntityId',
		id: 'mockVaultEntityId'
	} as unknown as SgVault;

	const mockBaseArgs: InternalTransactionArgs = {
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
		const removeOrderMockArgs: InternalTransactionArgs & { subgraphUrl: string; entity: SgOrder } =
			{
				subgraphUrl: 'https://api.example.com',
				txHash:
					'0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef' as `0x${string}`,
				chainId: 1,
				networkKey: 'ethereum',
				queryKey: '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
				entity: mockSgOrderEntity
			};

		const fullMockArgsForExpectation: InternalTransactionArgs & {
			awaitSubgraphConfig: AwaitSubgraphConfig;
		} = {
			...removeOrderMockArgs,
			awaitSubgraphConfig: {
				subgraphUrl: removeOrderMockArgs.subgraphUrl,
				txHash: removeOrderMockArgs.txHash,
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

			await manager.createRemoveOrderTransaction(removeOrderMockArgs);

			expect(TransactionStore).toHaveBeenCalledWith(
				{
					...fullMockArgsForExpectation,
					name: TransactionName.REMOVAL,
					errorMessage: 'Order removal failed.',
					successMessage: 'Order removed successfully.',
					queryKey: removeOrderMockArgs.queryKey,
					toastLinks: [
						{
							link: `/orders/${removeOrderMockArgs.networkKey}-${removeOrderMockArgs.queryKey}`,
							label: 'View Order'
						},
						{
							link: 'https://explorer.example.com/tx/0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
							label: 'View on explorer'
						}
					],
					config: mockWagmiConfig,
					awaitSubgraphConfig: {
						subgraphUrl: removeOrderMockArgs.subgraphUrl,
						txHash: removeOrderMockArgs.txHash,
						successMessage: 'Order removed successfully.',
						fetchEntityFn: getTransactionRemoveOrders,
						isSuccess: expect.any(Function)
					}
				},
				expect.any(Function),
				expect.any(Function)
			);

			const removeOrderCallArgs = vi.mocked(TransactionStore).mock.calls[0][0];
			const removeOrderIsSuccessFn = removeOrderCallArgs.awaitSubgraphConfig!.isSuccess;
			expect(
				removeOrderIsSuccessFn([
					{
						transaction: {
							id: 'tx1',
							from: '0xfrom',
							blockNumber: '123',
							timestamp: '1678886400'
						},
						order: { id: 'order1' }
					}
				] as SgRemoveOrderWithOrder[])
			).toBe(true);
			expect(removeOrderIsSuccessFn([] as SgRemoveOrderWithOrder[])).toBe(false);
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
		const withdrawMockArgs: InternalTransactionArgs & { subgraphUrl: string; entity: SgVault } = {
			subgraphUrl: 'https://api.example.com',
			txHash: '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef' as `0x${string}`,
			chainId: 1,
			networkKey: 'ethereum',
			queryKey: '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
			entity: mockSgVaultEntity // Added entity
		};

		const fullMockArgsForExpectation: InternalTransactionArgs & {
			awaitSubgraphConfig: AwaitSubgraphConfig;
		} = {
			...withdrawMockArgs, // Spreads withdrawMockArgs including entity
			awaitSubgraphConfig: {
				subgraphUrl: withdrawMockArgs.subgraphUrl,
				txHash: withdrawMockArgs.txHash,
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

			await manager.createWithdrawTransaction(withdrawMockArgs);

			expect(TransactionStore).toHaveBeenCalledWith(
				{
					...fullMockArgsForExpectation,
					name: TransactionName.WITHDRAWAL,
					errorMessage: 'Withdrawal failed.',
					successMessage: 'Withdrawal successful.',
					queryKey: withdrawMockArgs.queryKey,
					toastLinks: [
						{
							link: `/vaults/${withdrawMockArgs.networkKey}-${withdrawMockArgs.queryKey}`,
							label: 'View vault'
						},
						{
							link: 'https://explorer.example.com/tx/0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
							label: 'View on explorer'
						}
					],
					config: mockWagmiConfig,
					awaitSubgraphConfig: {
						subgraphUrl: withdrawMockArgs.subgraphUrl,
						txHash: withdrawMockArgs.txHash,
						successMessage: 'Withdrawal successful.',
						fetchEntityFn: getTransaction,
						isSuccess: expect.any(Function)
					}
				},
				expect.any(Function),
				expect.any(Function)
			);

			const withdrawCallArgs = vi.mocked(TransactionStore).mock.calls[0][0];
			const withdrawIsSuccessFn = withdrawCallArgs.awaitSubgraphConfig!.isSuccess;
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

			const testSpecificArgs: InternalTransactionArgs & { subgraphUrl: string; entity: SgVault } = {
				...withdrawMockArgs, // Use base withdraw args
				queryKey: '0xvaultid' // Override queryKey for this specific test
			};

			await manager.createWithdrawTransaction(testSpecificArgs);

			onSuccess!();

			expect(mockAddToast).toHaveBeenCalledWith({
				message: 'Withdrawal successful.',
				type: 'success',
				color: 'green',
				links: expect.any(Array)
			});
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
				subgraphUrl: 'https://api.example.com'
			});

			onSuccess!();

			expect(mockAddToast).toHaveBeenCalledWith({
				message: 'Order removed successfully.',
				type: 'success',
				color: 'green',
				links: expect.any(Array)
			});
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
				subgraphUrl: 'https://api.example.com'
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
				subgraphUrl: 'https://api.example.com'
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
		} as SgVault;
		const mockArgs: InternalTransactionArgs = {
			txHash: '0xapprovehash' as `0x${string}`,
			chainId: 1,
			networkKey: 'ethereum',
			queryKey: '0xvaultid'
		};

		beforeEach(() => {
			vi.mocked(getExplorerLink).mockResolvedValue('https://explorer.example.com/tx/0xapprovehash');
		});

		it('should create a transaction with correct parameters', async () => {
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
							link: `/vaults/${mockArgs.networkKey}-${mockArgs.queryKey}`,
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
	});

	describe('createDepositTransaction', () => {
		const mockEntity = {
			token: {
				symbol: 'TEST',
				decimals: '18'
			}
		} as SgVault;
		const mockArgs: InternalTransactionArgs & {
			amount: bigint;
			entity: SgVault;
			subgraphUrl: string;
		} = {
			subgraphUrl: 'https://api.example.com',
			txHash: '0xdeposithash' as `0x${string}`,
			chainId: 1,
			networkKey: 'ethereum',
			queryKey: '0xvaultid',
			entity: mockEntity,
			amount: 1000000000000000000n
		};

		beforeEach(() => {
			vi.mocked(getExplorerLink).mockResolvedValue('https://explorer.example.com/tx/0xdeposithash');
		});

		it('should create a transaction with correct parameters including formatted amount', async () => {
			const mockTransaction = { execute: vi.fn() };
			vi.mocked(TransactionStore).mockImplementation(
				() => mockTransaction as unknown as TransactionStore
			);

			const expectedReadableAmount = formatUnits(
				mockArgs.amount,
				Number(mockEntity.token.decimals)
			);

			await manager.createDepositTransaction(mockArgs);

			expect(TransactionStore).toHaveBeenCalledWith(
				{
					...mockArgs,
					name: `Depositing ${expectedReadableAmount} ${mockEntity.token.symbol}`,
					errorMessage: 'Deposit failed.',
					successMessage: 'Deposit successful.',
					queryKey: mockArgs.queryKey,
					toastLinks: [
						{
							link: `/vaults/${mockArgs.networkKey}-${mockArgs.queryKey}`,
							label: 'View vault'
						},
						{
							link: 'https://explorer.example.com/tx/0xdeposithash',
							label: 'View on explorer'
						}
					],
					config: mockWagmiConfig,
					awaitSubgraphConfig: {
						subgraphUrl: mockArgs.subgraphUrl,
						txHash: mockArgs.txHash,
						successMessage: 'Deposit successful.',
						fetchEntityFn: getTransaction,
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

			expect(mockAddToast).toHaveBeenCalledWith({
				message: 'Deposit successful.',
				type: 'success',
				color: 'green',
				links: expect.any(Array)
			});
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
});
