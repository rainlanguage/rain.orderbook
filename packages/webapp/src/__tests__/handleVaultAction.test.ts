import { describe, it, expect, vi, beforeEach, type MockedFunction } from 'vitest';
import { handleVaultAction } from '../lib/services/handleVaultAction';
import { handleDepositModal, handleWithdrawModal } from '$lib/services/modal';
import { invalidateTanstackQueries } from '@rainlanguage/ui-components';
import type { SgVault } from '@rainlanguage/orderbook';
import type { QueryClient } from '@tanstack/svelte-query';
import type { Hex } from 'viem';

vi.mock('$lib/services/modal', () => ({
	handleDepositModal: vi.fn(),
	handleWithdrawModal: vi.fn()
}));

vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
	const original = await importOriginal<typeof import('@rainlanguage/ui-components')>();
	return {
		...original,
		invalidateTanstackQueries: vi.fn()
	};
});

describe('handleVaultAction', () => {
	const mockQueryClient = {} as QueryClient;
	const mockVault = { id: 'mockVaultId' } as SgVault;
	const mockQueryKey = 'testQueryKey';
	const mockChainId = 1;
	const mockRpcUrl = 'http://localhost:8545';
	const mockSubgraphUrl = 'http://localhost:8000';
	const mockAccount = '0x123' as Hex;

	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('should call handleDepositModal when action is deposit', () => {
		handleVaultAction({
			vault: mockVault,
			action: 'deposit',
			queryClient: mockQueryClient,
			queryKey: mockQueryKey,
			chainId: mockChainId,
			rpcUrl: mockRpcUrl,
			subgraphUrl: mockSubgraphUrl,
			account: mockAccount
		});

		expect(handleDepositModal).toHaveBeenCalledOnce();
		expect(handleDepositModal).toHaveBeenCalledWith({
			open: true,
			args: {
				vault: mockVault,
				onSuccess: expect.any(Function),
				chainId: mockChainId,
				rpcUrl: mockRpcUrl,
				subgraphUrl: mockSubgraphUrl,
				account: mockAccount
			}
		});
		expect(handleWithdrawModal).not.toHaveBeenCalled();
	});

	it('should call handleWithdrawModal when action is withdraw', () => {
		handleVaultAction({
			vault: mockVault,
			action: 'withdraw',
			queryClient: mockQueryClient,
			queryKey: mockQueryKey,
			chainId: mockChainId,
			rpcUrl: mockRpcUrl,
			subgraphUrl: mockSubgraphUrl,
			account: mockAccount
		});

		expect(handleWithdrawModal).toHaveBeenCalledOnce();
		expect(handleWithdrawModal).toHaveBeenCalledWith({
			open: true,
			args: {
				vault: mockVault,
				onSuccess: expect.any(Function),
				chainId: mockChainId,
				rpcUrl: mockRpcUrl,
				subgraphUrl: mockSubgraphUrl,
				account: mockAccount
			}
		});
		expect(handleDepositModal).not.toHaveBeenCalled();
	});

	it('should call invalidateTanstackQueries on onSuccess for deposit', () => {
		handleVaultAction({
			vault: mockVault,
			action: 'deposit',
			queryClient: mockQueryClient,
			queryKey: mockQueryKey,
			chainId: mockChainId,
			rpcUrl: mockRpcUrl,
			subgraphUrl: mockSubgraphUrl,
			account: mockAccount
		});

		const SgVaultOnSuccess = (handleDepositModal as MockedFunction<typeof handleDepositModal>).mock
			.calls[0][0].args.onSuccess;
		SgVaultOnSuccess();

		expect(invalidateTanstackQueries).toHaveBeenCalledOnce();
		expect(invalidateTanstackQueries).toHaveBeenCalledWith(mockQueryClient, [mockQueryKey]);
	});

	it('should call invalidateTanstackQueries on onSuccess for withdraw', () => {
		handleVaultAction({
			vault: mockVault,
			action: 'withdraw',
			queryClient: mockQueryClient,
			queryKey: mockQueryKey,
			chainId: mockChainId,
			rpcUrl: mockRpcUrl,
			subgraphUrl: mockSubgraphUrl,
			account: mockAccount
		});

		const SgVaultOnSuccess = (handleWithdrawModal as MockedFunction<typeof handleWithdrawModal>)
			.mock.calls[0][0].args.onSuccess;
		SgVaultOnSuccess();

		expect(invalidateTanstackQueries).toHaveBeenCalledOnce();
		expect(invalidateTanstackQueries).toHaveBeenCalledWith(mockQueryClient, [mockQueryKey]);
	});
});
