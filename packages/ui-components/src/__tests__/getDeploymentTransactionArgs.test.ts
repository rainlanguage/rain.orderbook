import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { getDeploymentTransactionArgs } from '../lib/components/deployment/getDeploymentTransactionArgs';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

describe('getDeploymentTransactionArgs', () => {
	let guiInstance: DotrainOrderGui;

	beforeEach(() => {
		vi.clearAllMocks();
		guiInstance = new DotrainOrderGui();
		(DotrainOrderGui.prototype.getDeploymentTransactionArgs as Mock).mockResolvedValue({
			value: {
				chainId: 1,
				orderbookAddress: '0xorderbook',
				approvals: [{ token: '0x123', calldata: '0x1', symbol: 'TEST' }],
				deploymentCalldata: '0x1'
			}
		});
	});

	describe('successful cases', () => {
		it('should successfully return deployment transaction args', async () => {
			(DotrainOrderGui.prototype.generateApprovalCalldatas as Mock).mockResolvedValue({
				value: {
					Calldatas: [{ token: '0x123', amount: '1000' }]
				}
			});

			const result = await getDeploymentTransactionArgs(guiInstance, '0x123');

			expect(result).toEqual({
				approvals: [{ token: '0x123', calldata: '0x1', symbol: 'TEST' }],
				deploymentCalldata: '0x1',
				orderbookAddress: '0xorderbook',
				chainId: 1
			});
		});
	});

	describe('input validation errors', () => {
		it('should throw an error when gui.getDeploymentTransactionArgs returns an error object', async () => {
			// Create a new instance for this specific test
			const errorGuiInstance = new DotrainOrderGui();

			// Override the mock for this specific instance
			(DotrainOrderGui.prototype.getDeploymentTransactionArgs as Mock).mockResolvedValueOnce({
				error: {
					msg: 'Something went wrong with deployment'
				}
			});

			await expect(getDeploymentTransactionArgs(errorGuiInstance, '0x123')).rejects.toThrow(
				'Something went wrong with deployment'
			);
		});
	});
});
