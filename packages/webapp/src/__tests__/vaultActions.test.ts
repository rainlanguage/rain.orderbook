import { describe, it, expect, vi } from 'vitest';
import { handleVaultAction } from '$lib/services/vaultActions';
import { handleDepositOrWithdrawModal } from '$lib/services/modal';
import type { SgVault } from '@rainlanguage/orderbook';
import type { Hex } from 'viem';
import type { QueryClient } from '@tanstack/svelte-query';

vi.mock('$lib/services/modal', () => ({
	handleDepositOrWithdrawModal: vi.fn()
}));

describe('handleVaultAction', () => {
	const mockVault = {
		id: 'test-vault',
		token: {
			id: '0x123',
			address: '0x123',
			symbol: 'TEST',
			decimals: '18'
		},
		owner: '0x789',
		vaultId: 'test-vault',
		balance: '0',
		orderbook: '0xabc',
		createdAt: '0',
		updatedAt: '0',
		version: '1',
		ordersAsOutput: [],
		ordersAsInput: [],
		balanceChanges: []
	} as unknown as SgVault;

	const mockQueryClient = {
		invalidateQueries: vi.fn()
	} as unknown as QueryClient;

	const mockParams = {
		vault: mockVault,
		action: 'deposit' as const,
		chainId: 1,
		rpcUrl: 'http://test.rpc',
		subgraphUrl: 'http://test.subgraph',
		account: '0x456' as Hex,
		queryClient: mockQueryClient,
		vaultId: 'test-vault-id'
	};

	it('calls handleDepositOrWithdrawModal with correct parameters', () => {
		handleVaultAction(mockParams);

		expect(handleDepositOrWithdrawModal).toHaveBeenCalledWith({
			open: true,
			args: {
				vault: mockVault,
				onDepositOrWithdraw: expect.any(Function),
				action: 'deposit',
				chainId: 1,
				rpcUrl: 'http://test.rpc',
				subgraphUrl: 'http://test.subgraph',
				account: '0x456'
			}
		});
	});

	it('invalidates queries when onDepositOrWithdraw is called', () => {
		handleVaultAction(mockParams);

		const callArgs = (handleDepositOrWithdrawModal as any).mock.calls[0][0];
		const onDepositOrWithdraw = callArgs.args.onDepositOrWithdraw;

		onDepositOrWithdraw();

		expect(mockQueryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['test-vault-id'],
			exact: false,
			refetchType: 'all'
		});
	});

	it('handles withdraw action correctly', () => {
		handleVaultAction({
			...mockParams,
			action: 'withdraw'
		});

		expect(handleDepositOrWithdrawModal).toHaveBeenCalledWith({
			open: true,
			args: {
				vault: mockVault,
				onDepositOrWithdraw: expect.any(Function),
				action: 'withdraw',
				chainId: 1,
				rpcUrl: 'http://test.rpc',
				subgraphUrl: 'http://test.subgraph',
				account: '0x456'
			}
		});
	});
});
