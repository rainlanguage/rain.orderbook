import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import {
	getDeploymentTransactionArgs,
	AddOrderErrors
} from '../lib/components/deployment/getDeploymentTransactionArgs';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

describe('getDeploymentTransactionArgs', () => {
	let guiInstance: DotrainOrderGui;
	let mockGetDeploymentTransactionArgs: Mock;
	let mockGetNetworkKey: Mock;

	beforeEach(() => {
		vi.clearAllMocks();

		mockGetDeploymentTransactionArgs = vi.fn();
		mockGetNetworkKey = vi.fn();

		guiInstance = {
			getDeploymentTransactionArgs: mockGetDeploymentTransactionArgs,
			getNetworkKey: mockGetNetworkKey
		} as unknown as DotrainOrderGui;

		mockGetDeploymentTransactionArgs.mockResolvedValue({
			value: {
				chainId: 1,
				orderbookAddress: '0xorderbook',
				approvals: [{ token: '0x123', calldata: '0x1', symbol: 'TEST' }],
				deploymentCalldata: '0x1'
			}
		});

		mockGetNetworkKey.mockReturnValue({
			value: 'ethereum'
		});
	});

	describe('successful cases', () => {
		it('should successfully return deployment transaction args', async () => {
			const result = await getDeploymentTransactionArgs(guiInstance, '0x123');

			expect(mockGetNetworkKey).toHaveBeenCalled();
			expect(mockGetDeploymentTransactionArgs).toHaveBeenCalledWith('0x123');
			expect(result).toEqual({
				approvals: [{ token: '0x123', calldata: '0x1', symbol: 'TEST' }],
				deploymentCalldata: '0x1',
				orderbookAddress: '0xorderbook',
				chainId: 1,
				network: 'ethereum'
			});
		});
	});

	describe('input validation errors', () => {
		it('should throw NO_ACCOUNT_CONNECTED when wallet address is falsy', async () => {
			await expect(getDeploymentTransactionArgs(guiInstance, '')).rejects.toThrow(
				AddOrderErrors.NO_ACCOUNT_CONNECTED
			);

			await expect(
				getDeploymentTransactionArgs(guiInstance, null as unknown as string)
			).rejects.toThrow(AddOrderErrors.NO_ACCOUNT_CONNECTED);

			await expect(
				getDeploymentTransactionArgs(guiInstance, undefined as unknown as string)
			).rejects.toThrow(AddOrderErrors.NO_ACCOUNT_CONNECTED);
		});

		it('should throw ERROR_GETTING_NETWORK_KEY when getNetworkKey returns an error', async () => {
			mockGetNetworkKey.mockReturnValue({
				error: { msg: 'Network key error' }
			});

			await expect(getDeploymentTransactionArgs(guiInstance, '0x123')).rejects.toThrow(
				AddOrderErrors.ERROR_GETTING_NETWORK_KEY
			);
		});
	});

	describe('error handling', () => {
		it('should throw the error message when gui.getDeploymentTransactionArgs returns an error', async () => {
			const errorMessage = 'Custom error message';
			mockGetDeploymentTransactionArgs.mockResolvedValue({
				error: {
					msg: errorMessage
				}
			});

			await expect(getDeploymentTransactionArgs(guiInstance, '0x123')).rejects.toThrow(
				errorMessage
			);
		});
	});
});
