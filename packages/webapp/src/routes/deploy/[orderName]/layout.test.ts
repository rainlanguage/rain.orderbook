import { describe, it, expect, vi, beforeEach } from 'vitest';
import { load } from './+layout';
import { redirect } from '@sveltejs/kit';

vi.mock('@sveltejs/kit', () => ({
	redirect: vi.fn()
}));

describe('Layout load function', () => {
	const mockorderName = 'test-order';
	const mockDotrain = 'https://dotrain.example.com';
	const mockorderDetail = {
		name: 'Test Order',
		description: 'This is a test order',
		config: {}
	};

	const mockParent = vi.fn();

	beforeEach(() => {
		vi.resetAllMocks();

		mockParent.mockResolvedValue({
			registryDotrains: [
				{ name: mockorderName, dotrain: mockDotrain },
				{ name: 'other-order', dotrain: 'https://other.example.com' }
			],
			validOrders: [
				{
					name: mockorderName,
					details: mockorderDetail
				},
				{
					name: 'other-order',
					details: { name: 'Other', description: 'Other order', config: {} }
				}
			]
		});
	});

	it('should load order details successfully', async () => {
		const result = await load({
			params: { orderName: mockorderName },
			parent: mockParent
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
		} as any);

		expect(mockParent).toHaveBeenCalled();
		expect(result).toEqual({
			dotrain: mockDotrain,
			orderName: mockorderName,
			orderDetail: mockorderDetail,
			pageName: mockorderName
		});
	});

	it('should redirect if order name is not found in registryDotrains', async () => {
		try {
			await load({
				params: { orderName: 'non-existent-order' },
				parent: mockParent
				// eslint-disable-next-line @typescript-eslint/no-explicit-any
			} as any);
		} catch {
			expect(redirect).toHaveBeenCalledWith(307, '/deploy');
		}
	});

	it('should redirect if order details are not found in validOrders', async () => {
		mockParent.mockResolvedValue({
			registryDotrains: [{ name: 'incomplete-order', dotrain: 'https://incomplete.example.com' }],
			validOrders: [
				{ name: 'other-order', details: { name: 'Other', description: '', config: {} } }
			]
		});

		try {
			await load({
				params: { orderName: 'incomplete-order' },
				parent: mockParent
				// eslint-disable-next-line @typescript-eslint/no-explicit-any
			} as any);
			expect(redirect).toHaveBeenCalled();
		} catch {
			expect(redirect).toHaveBeenCalledWith(307, '/deploy');
		}
	});
});
