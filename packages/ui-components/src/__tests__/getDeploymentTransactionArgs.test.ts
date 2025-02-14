import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
	getDeploymentTransactionArgs,
	AddOrderErrors
} from '../lib/components/deployment/getDeploymentTransactionArgs';
import { getAccount } from '@wagmi/core';
import type { Config } from '@wagmi/core';
import type { DotrainOrderGui, OrderIO } from '@rainlanguage/orderbook/js_api';

// Mock wagmi/core
vi.mock('@wagmi/core', () => ({
	getAccount: vi.fn()
}));

describe('getDeploymentTransactionArgs', () => {
	let mockGui: DotrainOrderGui;
	let mockWagmiConfig: Config;
	let mockTokenOutputs: OrderIO[];

	beforeEach(() => {
		vi.clearAllMocks();

		// Mock GUI with successful responses
		mockGui = {
			generateApprovalCalldatas: vi.fn().mockResolvedValue([{ token: '0x123', amount: '1000' }]),
			generateDepositAndAddOrderCalldatas: vi.fn().mockResolvedValue({
				deposit: '0xdeposit',
				addOrder: '0xaddOrder'
			}),
			getCurrentDeployment: vi.fn().mockReturnValue({
				deployment: {
					order: {
						network: { 'chain-id': 1 },
						orderbook: { address: '0xorderbook' }
					}
				}
			}),
			getTokenInfo: vi.fn().mockResolvedValue({
				address: '0x123',
				symbol: 'TEST'
			})
		} as unknown as DotrainOrderGui;

		mockWagmiConfig = {} as Config;
		(getAccount as any).mockReturnValue({ address: '0xuser' });

		mockTokenOutputs = [{ token: { key: 'token1' } }] as OrderIO[];
	});

	describe('successful cases', () => {
		it('should successfully return deployment transaction args', async () => {
			const result = await getDeploymentTransactionArgs(mockGui, mockWagmiConfig, mockTokenOutputs);

			expect(result).toEqual({
				approvals: [{ token: '0x123', amount: '1000', symbol: 'TEST' }],
				deploymentCalldata: {
					deposit: '0xdeposit',
					addOrder: '0xaddOrder'
				},
				orderbookAddress: '0xorderbook',
				chainId: 1
			});

			expect(mockGui.generateApprovalCalldatas).toHaveBeenCalledWith('0xuser');
			expect(mockGui.generateDepositAndAddOrderCalldatas).toHaveBeenCalled();
		});
	});

	describe('input validation errors', () => {
		it('should throw MISSING_GUI when GUI is null', async () => {
			await expect(
				getDeploymentTransactionArgs(null, mockWagmiConfig, mockTokenOutputs)
			).rejects.toThrow(AddOrderErrors.MISSING_GUI);
		});

		it('should throw MISSING_CONFIG when wagmiConfig is undefined', async () => {
			await expect(
				getDeploymentTransactionArgs(mockGui, undefined, mockTokenOutputs)
			).rejects.toThrow(AddOrderErrors.MISSING_CONFIG);
		});

		it('should throw NO_WALLET when wallet address is not found', async () => {
			(getAccount as any).mockReturnValue({ address: null });
			await expect(
				getDeploymentTransactionArgs(mockGui, mockWagmiConfig, mockTokenOutputs)
			).rejects.toThrow(AddOrderErrors.NO_WALLET);
		});
	});

	describe('deployment errors', () => {
		it('should throw INVALID_CHAIN_ID when chain ID is missing', async () => {
			mockGui.getCurrentDeployment = vi.fn().mockReturnValue({
				deployment: {
					order: {
						network: {},
						orderbook: { address: '0xorderbook' }
					}
				}
			});

			await expect(
				getDeploymentTransactionArgs(mockGui, mockWagmiConfig, mockTokenOutputs)
			).rejects.toThrow(AddOrderErrors.INVALID_CHAIN_ID);
		});

		it('should throw MISSING_ORDERBOOK when orderbook address is missing', async () => {
			mockGui.getCurrentDeployment = vi.fn().mockReturnValue({
				deployment: {
					order: {
						network: { 'chain-id': 1 },
						orderbook: {}
					}
				}
			});

			await expect(
				getDeploymentTransactionArgs(mockGui, mockWagmiConfig, mockTokenOutputs)
			).rejects.toThrow(AddOrderErrors.MISSING_ORDERBOOK);
		});
	});

	describe('approval and calldata errors', () => {
		it('should throw APPROVAL_FAILED when generateApprovalCalldatas fails', async () => {
			mockGui.generateApprovalCalldatas = vi.fn().mockRejectedValue(new Error('Approval error'));

			await expect(
				getDeploymentTransactionArgs(mockGui, mockWagmiConfig, mockTokenOutputs)
			).rejects.toThrow(`${AddOrderErrors.APPROVAL_FAILED}: Approval error`);
		});

		it('should throw DEPLOYMENT_FAILED when generateDepositAndAddOrderCalldatas fails', async () => {
			mockGui.generateDepositAndAddOrderCalldatas = vi
				.fn()
				.mockRejectedValue(new Error('Deployment error'));

			await expect(
				getDeploymentTransactionArgs(mockGui, mockWagmiConfig, mockTokenOutputs)
			).rejects.toThrow(`${AddOrderErrors.DEPLOYMENT_FAILED}: Deployment error`);
		});
	});

	describe('token info errors', () => {
		it('should throw TOKEN_INFO_FAILED when token key is missing', async () => {
			const invalidTokenOutputs = [{ token: {} }] as OrderIO[];

			await expect(
				getDeploymentTransactionArgs(mockGui, mockWagmiConfig, invalidTokenOutputs)
			).rejects.toThrow(`${AddOrderErrors.TOKEN_INFO_FAILED}: Token key is missing`);
		});

		it('should throw TOKEN_INFO_FAILED when getTokenInfo fails', async () => {
			mockGui.getTokenInfo = vi.fn().mockRejectedValue(new Error('Token info error'));

			await expect(
				getDeploymentTransactionArgs(mockGui, mockWagmiConfig, mockTokenOutputs)
			).rejects.toThrow(`${AddOrderErrors.TOKEN_INFO_FAILED}: Token info error`);
		});

		it('should throw TOKEN_INFO_FAILED when token info is not found for approval', async () => {
			mockGui.getTokenInfo = vi.fn().mockResolvedValue({
				address: '0x456', // Different address than the approval token
				symbol: 'TEST'
			});

			await expect(
				getDeploymentTransactionArgs(mockGui, mockWagmiConfig, mockTokenOutputs)
			).rejects.toThrow(
				`${AddOrderErrors.TOKEN_INFO_FAILED}: Token info not found for address: 0x123`
			);
		});
	});
});
