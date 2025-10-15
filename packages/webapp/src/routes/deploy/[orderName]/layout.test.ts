import { describe, it, expect } from 'vitest';
import { load } from './+layout';

describe('Layout load function', () => {
	it('returns orderName and pageName when params include orderName', async () => {
		const result = await load({
			params: { orderName: 'test-order' }
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
		} as any);

		expect(result).toEqual({
			orderName: 'test-order',
			pageName: 'test-order'
		});
	});

	it('returns undefined when params.orderName is missing', async () => {
		const result = await load({
			params: {}
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
		} as any);

		expect(result).toEqual({
			orderName: undefined,
			pageName: undefined
		});
	});
});
