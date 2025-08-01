import {
	RaindexFilterStore as RaindexFilterStoreWasm,
	type GetVaultsFilters,
	type WasmEncodedResult
} from '@rainlanguage/orderbook';
import { GetVaultsFilterBuilder } from './GetVaultsFilterBuilder.js';

export type GetVaultsFilterUpdateCallback = (
	builder: GetVaultsFilterBuilder
) => GetVaultsFilterBuilder;

/**
 * Helper function to unwrap WASM encoded results
 */
function unwrapWasmResult<T>(result: WasmEncodedResult<T>): T {
	if ('error' in result && result.error) {
		throw new Error(`WASM error: ${result.error.readableMsg || result.error}`);
	}
	if ('value' in result) {
		return result.value;
	}
	throw new Error('Invalid WASM result format');
}

/**
 * TypeScript wrapper around the WASM RaindexFilterStore that provides
 * a clean fluent API for updating filters with automatic persistence.
 *
 * This wrapper solves the TypeScript typing issues with WASM generics
 * by providing a pure TypeScript builder pattern interface.
 */
export class RaindexFilterStore {
	private wasmStore: RaindexFilterStoreWasm;

	constructor() {
		const wasmStoreResult = RaindexFilterStoreWasm.create();
		this.wasmStore = unwrapWasmResult(wasmStoreResult);
	}

	/**
	 * Update vault filters using a fluent builder API
	 *
	 * @example
	 * ```typescript
	 * const updated = store.update(builder =>
	 *   builder.setOwners([address]).setHideZeroBalance(true)
	 * );
	 * ```
	 */
	updateVaults(callback: GetVaultsFilterUpdateCallback): RaindexFilterStore {
		try {
			// Get current filters from WASM store
			const currentFiltersResult = this.wasmStore.getVaults();
			const currentFilters = unwrapWasmResult<GetVaultsFilters>(currentFiltersResult);

			// Create builder with current state
			const builder = new GetVaultsFilterBuilder(currentFilters);

			// Let user update the builder
			const updatedBuilder = callback(builder);

			// Apply changes through WASM (this auto-saves to localStorage and vault URL params)
			const newWasmStoreResult = this.wasmStore.updateVaults(updatedBuilder.build());
			this.wasmStore = unwrapWasmResult<RaindexFilterStoreWasm>(newWasmStoreResult);

			return this;
		} catch (error) {
			throw new Error(`Filter update failed: ${error}`);
		}
	}

	/**
	 * Directly set vault filters, replacing the current filters.
	 * @param filters The new vault filters to set.
	 */
	setVaults(filters: GetVaultsFilters): RaindexFilterStore {
		try {
			const result = this.wasmStore.setVaults(filters);
			this.wasmStore = unwrapWasmResult<RaindexFilterStoreWasm>(result);
			return this;
		} catch (error) {
			throw new Error(`Failed to set vault filters: ${error}`);
		}
	}

	/**
	 * Get current vault filters
	 */
	getVaultsFilters(): GetVaultsFilters {
		try {
			const result = this.wasmStore.getVaults();
			return unwrapWasmResult<GetVaultsFilters>(result);
		} catch (error) {
			throw new Error(`Failed to get filters: ${error}`);
		}
	}

	/**
	 * Manually save filters to persistent storage
	 * (Note: updates through .update() auto-save, this is for manual saves)
	 */
	save(): void {
		try {
			const result = this.wasmStore.save();
			unwrapWasmResult<void>(result);
		} catch (error) {
			throw new Error(`Failed to save filters: ${error}`);
		}
	}

	/**
	 * Reload filters from persistent storage
	 */
	load(): RaindexFilterStore {
		try {
			const newWasmStoreResult = this.wasmStore.load();
			this.wasmStore = unwrapWasmResult<RaindexFilterStoreWasm>(newWasmStoreResult);
			return this;
		} catch (error) {
			throw new Error(`Failed to load filters: ${error}`);
		}
	}
}
