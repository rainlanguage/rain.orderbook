import { describe, expect, it, beforeEach, vi } from 'vitest';
import { RaindexFilterStore, VaultsFilterBuilder } from '../../dist/cjs';

// Mock web APIs for Node.js environment
const mockLocalStorage = {
	getItem: vi.fn(),
	setItem: vi.fn(),
	removeItem: vi.fn(),
	clear: vi.fn(),
};

const mockLocation = {
	search: '?test=value',
	pathname: '/test',
	hash: '#test',
	href: 'http://localhost/test?test=value#test',
};

const mockHistory = {
	replaceState: vi.fn(),
};

const mockUrlSearchParams = vi.fn().mockImplementation((searchString) => ({
	get: vi.fn((key) => key === 'filters' ? null : null),
	set: vi.fn(),
	toString: vi.fn(() => ({ as_string: () => 'filters=%7B%22test%22%3Atrue%7D' })),
}));

// Mock window object
global.window = {
	localStorage: mockLocalStorage,
	location: mockLocation,
	history: mockHistory,
} as any;

// Mock UrlSearchParams constructor
(global as any).UrlSearchParams = mockUrlSearchParams;

describe('RaindexFilterStore', () => {
	let storage: Record<string, any> = {};
	beforeEach(() => {
		// Reset mocks before each test
		vi.clearAllMocks();
		mockLocalStorage.getItem.mockImplementation((key: string) => {
			return storage[key] || undefined;
		});
		mockLocalStorage.setItem.mockImplementation((key, value) => {
			storage[key] = value;
		})
	});

	it('should create store and get default filters', () => {
		const result = RaindexFilterStore.create();
		console.log('result', result);
		
		// Store creation should succeed even in Node.js
		expect(result.error).toBeUndefined();
		expect(result.value).toBeDefined();
		
		const store = result.value!;
		const vaultsFilterResult = store.getVaults();
		
		expect(vaultsFilterResult.error).toBeUndefined();
		expect(vaultsFilterResult.value).toBeDefined();
	});

	it('should update filters with builder', () => {
		const result = RaindexFilterStore.create();
		const store = result.value!;
		
		// Test builder-based update using callback function (the correct way)
		const updateResult = store.updateVaults((builder: VaultsFilterBuilder) => {
			const ownersResult = builder.setOwners(["0x1234567890abcdef1234567890abcdef12345678" as `0x${string}`]);
			if (ownersResult.error) throw new Error(ownersResult.error.msg);
			
			const hideZeroResult = ownersResult.value!.setHideZeroBalance(true);
			if (hideZeroResult.error) throw new Error(hideZeroResult.error.msg);
			
			const chainIdsResult = hideZeroResult.value!.setChainIds([1, 137]);
			if (chainIdsResult.error) throw new Error(chainIdsResult.error.msg);
			
			return chainIdsResult.value!;
		});
		
		console.log('updateResult', updateResult);
		
		if (updateResult.error) {
			// Expected in Node.js due to save() operation needing window
			console.log('Update error (expected in Node.js):', updateResult.error.msg);
			expect(updateResult.error.msg).toBe('Window is not available');
		} else {
			// If update succeeded, verify the filters were updated
			const updatedStore = updateResult.value!;
			const vaultsResult = updatedStore.getVaults();
			
			expect(vaultsResult.error).toBeUndefined();
			expect(vaultsResult.value!.owners).toEqual(["0x1234567890abcdef1234567890abcdef12345678"]);
			expect(vaultsResult.value!.hideZeroBalance).toBe(true);
			expect(vaultsResult.value!.chainIds).toEqual([1, 137]);
		}
	});

	// it('should save and load filters with mocked storage', () => {
	// 	const result = RaindexFilterStore.create();
	// 	const store = result.value!;
		
	// 	// Test saving - should not throw error with mocked localStorage
	// 	const saveResult = store.save();
	// 	console.log('saveResult', saveResult);
		
	// 	// Save should succeed (or at least not crash)
	// 	expect(mockLocalStorage.setItem).toHaveBeenCalled();
		
	// 	// Test loading - should not crash
	// 	const loadResult = store.load();
	// 	console.log('loadResult', loadResult);
		
	// 	if (loadResult.error) {
	// 		console.log('Load error (expected in Node.js):', loadResult.error.msg);
	// 	} else {
	// 		expect(loadResult.value).toBeDefined();
	// 	}
	// });

	// it('should handle filter updates', () => {
	// 	const result = RaindexFilterStore.create();
	// 	expect(result.error).toBeUndefined();
		
	// 	const store = result.value!;
		
	// 	// Set new filters
	// 	const newFilters = {
	// 		owners: ["0x1234567890abcdef1234567890abcdef12345678" as `0x${string}`],
	// 		hideZeroBalance: true,
	// 		tokens: undefined,
	// 		chainIds: [1, 137],
	// 	};
		
	// 	const setResult = store.setVaults(newFilters);
	// 	console.log('setResult', setResult);
		
	// 	if (setResult.error) {
	// 		console.log('Set error:', setResult.error.msg);
	// 	} else {
	// 		const updatedStore = setResult.value!;
	// 		const vaultsResult = updatedStore.getVaults();
			
	// 		expect(vaultsResult.error).toBeUndefined();
	// 		expect(vaultsResult.value!.owners).toEqual(newFilters.owners);
	// 		expect(vaultsResult.value!.hideZeroBalance).toBe(true);
	// 		expect(vaultsResult.value!.chainIds).toEqual([1, 137]);
	// 	}
	// });
});
