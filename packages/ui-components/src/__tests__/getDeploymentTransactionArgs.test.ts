import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import {
	getDeploymentTransactionArgs,
	AddOrderErrors
} from '../lib/components/deployment/getDeploymentTransactionArgs';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import type { Hex } from 'viem';

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

			const result = await getDeploymentTransactionArgs(guiInstance, '0xuser');

			expect(result).toEqual({
				approvals: [{ token: '0x123', calldata: '0x1', symbol: 'TEST' }],
				deploymentCalldata: '0x1',
				orderbookAddress: '0xorderbook',
				chainId: 1
			});
		});
	});

	describe('error handling', () => {
		it('should throw when gui.getDeploymentTransactionArgs returns an error object', async () => {
			(DotrainOrderGui.prototype.getDeploymentTransactionArgs as Mock).mockResolvedValue({
				error: { msg: 'Something went wrong with deployment' }
			});

			await expect(getDeploymentTransactionArgs(guiInstance, '0xuser')).rejects.toThrow(
				'Something went wrong with deployment'
			);
		});
	});
});
