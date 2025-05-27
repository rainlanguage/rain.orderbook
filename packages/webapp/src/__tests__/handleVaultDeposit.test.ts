import { describe, it, expect, vi, beforeEach } from 'vitest';
import { handleVaultDeposit } from '../lib/services/handleVaultDeposit';
import { handleDepositModal, handleWithdrawModal } from '$lib/services/modal';
import type { SgVault } from '@rainlanguage/orderbook';
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

describe('handleVaultDeposit', () => {
	const mockVault = { id: 'mockVaultId' } as SgVault;
	const mockChainId = 1;
	const mockRpcUrl = 'http://localhost:8545';
	const mockSubgraphUrl = 'http://localhost:8000';
	const mockAccount = '0x123' as Hex;

	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('should call handleDepositModal when action is deposit', () => {
		handleVaultDeposit({
			vault: mockVault,
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
				chainId: mockChainId,
				rpcUrl: mockRpcUrl,
				subgraphUrl: mockSubgraphUrl,
				account: mockAccount
			}
		});
		expect(handleWithdrawModal).not.toHaveBeenCalled();
	});
});
