import { describe, expect, it } from 'vitest';
import { DefaultWebFilterStore } from '../../dist/cjs';

describe('DefaultWebFilterStore', () => {
	// Since DefaultWebFilterStore requires browser environment,
	// we'll test that it fails gracefully in Node.js

	// TODO: Run tests in a browser-like environment and write tests for the store methods
	it('should create a store instance (expect window error in Node.js)', async () => {
		const result = DefaultWebFilterStore.create('test-filters');
		// In Node.js environment, we expect "Window is not available" error
		expect(result.error).toBeDefined();
		expect(result.error!.msg).toBe('Window is not available');
		expect(result.value).toBeUndefined();
	});
});
