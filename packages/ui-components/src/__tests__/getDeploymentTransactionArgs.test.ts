import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import {
	getDeploymentTransactionArgs,
	AddOrderErrors
} from '../lib/components/deployment/getDeploymentTransactionArgs';
import { getAccount } from '@wagmi/core';
import type { Config } from '@wagmi/core';
import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

// Mock wagmi/core
vi.mock('@wagmi/core', () => ({
	getAccount: vi.fn()
}));

describe('getDeploymentTransactionArgs', () => {
	let mockGui: DotrainOrderGui;
	let mockWagmiConfig: Config;

	beforeEach(() => {
		vi.clearAllMocks();

		// Mock GUI with successful responses
		mockGui = {
			getDeploymentTransactionArgs: vi.fn().mockResolvedValue({
				chainId: 1,
				orderbookAddress: '0xorderbook',
				approvals: [{ token: '0x123', calldata: '0x1', symbol: 'TEST' }],
				deploymentCalldata: '0x1'
			})
		} as unknown as DotrainOrderGui;

		mockWagmiConfig = {} as Config;
		(getAccount as Mock).mockReturnValue({ address: '0xuser' });
	});

	describe('successful cases', () => {
		it('should successfully return deployment transaction args', async () => {
			mockGui.generateApprovalCalldatas = vi.fn().mockResolvedValue({
				Calldatas: [{ token: '0x123', amount: '1000' }]
			});
			const result = await getDeploymentTransactionArgs(mockGui, mockWagmiConfig);

			expect(result).toEqual({
				approvals: [{ token: '0x123', calldata: '0x1', symbol: 'TEST' }],
				deploymentCalldata: '0x1',
				orderbookAddress: '0xorderbook',
				chainId: 1
			});
		});
	});

	describe('input validation errors', () => {
		it('should throw MISSING_GUI when GUI is null', async () => {
			await expect(getDeploymentTransactionArgs(null, mockWagmiConfig)).rejects.toThrow(
				AddOrderErrors.MISSING_GUI
			);
		});

		it('should throw MISSING_CONFIG when wagmiConfig is undefined', async () => {
			await expect(getDeploymentTransactionArgs(mockGui, undefined)).rejects.toThrow(
				AddOrderErrors.MISSING_CONFIG
			);
		});

		it('should throw NO_WALLET when wallet address is not found', async () => {
			(getAccount as Mock).mockReturnValue({ address: null });
			await expect(getDeploymentTransactionArgs(mockGui, mockWagmiConfig)).rejects.toThrow(
				AddOrderErrors.NO_WALLET
			);
		});
	});
});
