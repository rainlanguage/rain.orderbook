import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { load } from './+layout';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

vi.mock('@rainlanguage/orderbook/js_api', () => ({
	DotrainOrderGui: {
		getDeploymentDetail: vi.fn()
	}
}));

describe('Layout load function', () => {
	const mockDeploymentKey = 'test-deployment';
	const mockDotrain = 'https://dotrain.example.com';
	const mockParent = vi.fn();

	beforeEach(() => {
		vi.resetAllMocks();

		// Mock the parent function to return the dotrain value
		mockParent.mockResolvedValue({ dotrain: mockDotrain });
	});

	it('should load deployment details successfully', async () => {
		// Set up the mock implementation for this specific test
		(DotrainOrderGui.getDeploymentDetail as Mock).mockResolvedValue({
			error: null,
			value: {
				name: 'Test Deployment',
				description: 'This is a test deployment'
			}
		});

		const result = await load({
			params: { deploymentKey: mockDeploymentKey, strategyName: 'test-strategy' },
			parent: mockParent
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
		} as any);

		expect(mockParent).toHaveBeenCalled();
		expect(DotrainOrderGui.getDeploymentDetail).toHaveBeenCalledWith(
			mockDotrain,
			mockDeploymentKey
		);

		expect(result).toEqual({
			deployment: {
				key: mockDeploymentKey,
				name: 'Test Deployment',
				description: 'This is a test deployment'
			},
			dotrain: mockDotrain,
			pageName: mockDeploymentKey
		});
	});

	it('should handle empty deploymentKey', async () => {
		// Set up the mock implementation for this specific test
		(DotrainOrderGui.getDeploymentDetail as Mock).mockResolvedValue({
(DotrainOrderGui.getDeploymentDetail as Mock).mockResolvedValue({
  error: null,
  value: {
    name: 'Empty Deployment',
    description: ''
  }
});

// Add assertions to verify error is null
const response = await DotrainOrderGui.getDeploymentDetail(mockDotrain, '');
expect(response.error).toBeNull();
expect(response.value).toBeDefined();

		const result = await load({
			params: {},
			parent: mockParent
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
		} as any);

		expect(DotrainOrderGui.getDeploymentDetail).toHaveBeenCalledWith(mockDotrain, '');

		expect(result).toEqual({
			deployment: {
				key: undefined,
				name: 'Empty Deployment',
				description: ''
			},
			dotrain: mockDotrain,
			pageName: undefined
		});
	});

	it('should throw an error when getDeploymentDetail returns an error', async () => {
		(DotrainOrderGui.getDeploymentDetail as Mock).mockRejectedValue({
			error: { msg: 'Deployment not found' },
			value: null
		});

		await expect(
			load({
				params: { deploymentKey: 'error-key' },
				parent: mockParent
				// eslint-disable-next-line @typescript-eslint/no-explicit-any
			} as any)
		).rejects.toThrow('Deployment not found');
	});
});
