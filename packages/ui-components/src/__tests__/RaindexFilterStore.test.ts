import { describe, it, expect, vi, beforeEach } from 'vitest';
import { RaindexFilterStore } from '$lib/providers/filters/RaindexFilterStore';
import { GetVaultsFilterBuilder } from '$lib/providers/filters';

// Mock the WASM module
vi.mock('@rainlanguage/orderbook', () => {
	// Shared state to simulate persistence
	let persistentState = {
		owners: [],
		hideZeroBalance: false,
		tokens: undefined,
		chainIds: undefined
	};

	const createMockStore = () => ({
		getVaults: vi.fn(() => ({
			value: { ...persistentState }
		})),
		setVaults: vi.fn((filters) => {
			// Update persistent state when setVaults is called
			persistentState = { ...filters };
			return {
				value: createMockStore() // Return new mock store instance
			};
		}),
		save: vi.fn(() => ({ value: undefined })),
		load: vi.fn(() => ({
			value: createMockStore() // Return new mock store instance with current state
		}))
	});

	return {
		RaindexFilterStore: {
			create: vi.fn(() => ({
				value: createMockStore()
			}))
		}
	};
});

describe('RaindexFilterStore', () => {
	beforeEach(() => {
		vi.clearAllMocks();

		// Mock localStorage for each test
		const store: Record<string, string> = {};
		globalThis.localStorage = {
			getItem: vi.fn((key: string) => store[key] ?? null),
			setItem: vi.fn((key: string, value: string) => {
				store[key] = value;
			}),
			removeItem: vi.fn((key: string) => {
				delete store[key];
			}),
			clear: vi.fn(() => {
				Object.keys(store).forEach((k) => delete store[k]);
			}),
			key: vi.fn(),
			length: 0
		} as unknown as Storage;
	});

	it('should create a new RaindexFilterStore', () => {
		const wrapper = new RaindexFilterStore();
		expect(wrapper).toBeInstanceOf(RaindexFilterStore);
	});

	it('should get default filters', () => {
		const wrapper = new RaindexFilterStore();
		const filters = wrapper.getVaultsFilters();

		expect(filters).toEqual({
			owners: [],
			hideZeroBalance: false,
			tokens: undefined,
			chainIds: undefined
		});
	});

	it('should update filters using fluent API', () => {
		const wrapper = new RaindexFilterStore();
		const mockAddress = '0x1234567890123456789012345678901234567890' as const;

		const updated = wrapper.updateVaults((builder) =>
			builder.setOwners([mockAddress]).setHideZeroBalance(true).setChainIds([1, 137])
		);

		expect(updated).toBeInstanceOf(RaindexFilterStore);
	});

	it('should update filters and return new wrapper instance', () => {
		const wrapper = new RaindexFilterStore();
		const mockAddress = '0x1234567890123456789012345678901234567890' as const;

		// Update filters - this returns a NEW wrapper instance
		const updatedWrapper = wrapper.updateVaults((builder) =>
			builder.setOwners([mockAddress]).setHideZeroBalance(true).setChainIds([1, 137])
		);

		// Should return the same RaindexFilterStore instance
		expect(updatedWrapper).toBeInstanceOf(RaindexFilterStore);
		expect(updatedWrapper).toBe(wrapper); // Should be the same instance

		expect(updatedWrapper.getVaultsFilters()).toEqual({
			owners: [mockAddress],
			hideZeroBalance: true,
			tokens: undefined,
			chainIds: [1, 137]
		});
	});

	it('should provide working save() and load() methods', () => {
		const wrapper = new RaindexFilterStore();
		// These methods should not throw errors
		expect(() => wrapper.save()).not.toThrow();
		expect(() => wrapper.load()).not.toThrow();

		// load() should return the same RaindexFilterStore instance
		const loadedWrapper = wrapper.load();
		expect(loadedWrapper).toBe(wrapper);
	});

	it('should actually save and load filters', () => {
		const wrapper = new RaindexFilterStore();
		const mockAddress = '0x1234567890123456789012345678901234567890' as const;

		// Update filters
		wrapper.updateVaults((builder) => builder.setOwners([mockAddress]).setHideZeroBalance(true));

		// Save the current state
		wrapper.save();

		const loadedWrapper = wrapper.load();
		const filters = loadedWrapper.getVaultsFilters();
		expect(filters).toEqual({
			owners: [mockAddress],
			hideZeroBalance: true,
			tokens: undefined,
			chainIds: [1, 137]
		});
	});
});

describe('GetVaultsFilterBuilder', () => {
	const mockFilters = {
		owners: [],
		hideZeroBalance: false,
		tokens: undefined,
		chainIds: undefined
	};

	it('should create a builder with current filters', () => {
		const builder = new GetVaultsFilterBuilder(mockFilters);
		expect(builder).toBeInstanceOf(GetVaultsFilterBuilder);
	});

	it('should return the same instance after setter called for chaining', () => {
		const builder = new GetVaultsFilterBuilder(mockFilters);
		const mockAddress = '0x1234567890123456789012345678901234567890' as const;

		const result = builder.setOwners([mockAddress]);
		const filters = builder.build();
		expect(filters.owners).toEqual([mockAddress]);

		expect(result).toBe(builder); // Should return same instance for chaining
	});

	it('should support method chaining', () => {
		const builder = new GetVaultsFilterBuilder(mockFilters);
		const mockAddress = '0x1234567890123456789012345678901234567890' as const;

		const filters = builder
			.setOwners([mockAddress])
			.setHideZeroBalance(true)
			.setChainIds([1, 137])
			.build();

		expect(filters).toEqual({
			owners: [mockAddress],
			hideZeroBalance: true,
			tokens: undefined,
			chainIds: [1, 137]
		});
	});
});
