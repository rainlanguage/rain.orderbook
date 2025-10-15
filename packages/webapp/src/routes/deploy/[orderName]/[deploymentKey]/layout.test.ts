import { describe, it, expect } from 'vitest';
import { load } from './+layout';

describe('Layout load function', () => {
	it('returns pageName matching deploymentKey when provided', async () => {
		const result = await load({
			params: { deploymentKey: 'test-deployment' }
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
		} as any);

		expect(result).toEqual({
			pageName: 'test-deployment'
		});
	});

	it('returns undefined when deploymentKey param is missing', async () => {
		const result = await load({
			params: {}
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
		} as any);

		expect(result).toEqual({
			pageName: undefined
		});
	});
});
