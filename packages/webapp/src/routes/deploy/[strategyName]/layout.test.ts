import { describe, it, expect, vi, beforeEach } from 'vitest';
import { load } from './+layout';
import { redirect } from '@sveltejs/kit';

vi.mock('@sveltejs/kit', () => ({
	redirect: vi.fn()
}));

describe('Layout load function', () => {
	const mockStrategyName = 'test-strategy';
	const mockDotrain = 'https://dotrain.example.com';
	const mockStrategyDetail = {
		name: 'Test Strategy',
		description: 'This is a test strategy',
		config: {}
	};

	const mockParent = vi.fn();

	beforeEach(() => {
		vi.resetAllMocks();

		mockParent.mockResolvedValue({
			registryDotrains: [
				{ name: mockStrategyName, dotrain: mockDotrain },
				{ name: 'other-strategy', dotrain: 'https://other.example.com' }
			],
			validStrategies: [
				{
					name: mockStrategyName,
					details: mockStrategyDetail
				},
				{
					name: 'other-strategy',
					details: { name: 'Other', description: 'Other strategy', config: {} }
				}
			]
		});
	});

	it('should load strategy details successfully', async () => {
		const result = await load({
			params: { strategyName: mockStrategyName },
			parent: mockParent
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
		} as any);

		expect(mockParent).toHaveBeenCalled();
		expect(result).toEqual({
			dotrain: mockDotrain,
			strategyName: mockStrategyName,
			strategyDetail: mockStrategyDetail,
			pageName: mockStrategyName
		});
	});

	it('should redirect if strategy name is not found in registryDotrains', async () => {
		try {
			await load({
				params: { strategyName: 'non-existent-strategy' },
				parent: mockParent
				// eslint-disable-next-line @typescript-eslint/no-explicit-any
			} as any);
		} catch {
			expect(redirect).toHaveBeenCalledWith(307, '/deploy');
		}
	});

	it('should redirect if strategy details are not found in validStrategies', async () => {
		mockParent.mockResolvedValue({
			registryDotrains: [
				{ name: 'incomplete-strategy', dotrain: 'https://incomplete.example.com' }
			],
			validStrategies: [
				{ name: 'other-strategy', details: { name: 'Other', description: '', config: {} } }
			]
		});

		try {
			await load({
				params: { strategyName: 'incomplete-strategy' },
				parent: mockParent
				// eslint-disable-next-line @typescript-eslint/no-explicit-any
			} as any);
			expect(redirect).toHaveBeenCalled();
		} catch {
			expect(redirect).toHaveBeenCalledWith(307, '/deploy');
		}
	});
});
