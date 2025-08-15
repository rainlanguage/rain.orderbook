import { describe, expect, it } from 'vitest';
import { RaindexFilterStore, VaultsFilterBuilder } from '../../dist/cjs';

describe('RaindexFilterStore', () => {
	it('should create store and get default filters', () => {
		const result = RaindexFilterStore.create();

		// Store creation should succeed even in Node.js
		expect(result.error).toBeUndefined();
		expect(result.value).toBeDefined();

		const store = result.value!;
		const vaultsFilterResult = store.getVaults();

		expect(vaultsFilterResult.error).toBeUndefined();
		expect(vaultsFilterResult.value).toBeDefined();
	});

	it('should handle set filter without calling local Storage', () => {
		const result = RaindexFilterStore.create();
		const store = result.value!;

		const builder = VaultsFilterBuilder.new().value!;
		const newFilters = builder
			.setOwners(['0x1234567890abcdef1234567890abcdef12345678' as `0x${string}`])
			.value!.setHideZeroBalance(true)
			.value!.setChainIds([1, 137])
			.value!.build().value!;

		const store2 = store.setVaults(newFilters).value!;
		const updatedFilters = store2.getVaults();

		expect(updatedFilters.error).toBeUndefined();
		expect(updatedFilters.value).toEqual(newFilters);
	});
});
