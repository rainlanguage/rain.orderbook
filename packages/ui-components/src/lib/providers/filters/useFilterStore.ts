import { getContext } from 'svelte';
import type { Readable } from 'svelte/store';
import type { RaindexFilterStore } from './RaindexFilterStore';
import { FILTER_STORE_CONTEXT } from './FilterStoreProvider.svelte';

/**
 * Hook for accessing the filter store from any Svelte component.
 * Must be used within a component that is wrapped by FilterStoreProvider.
 *
 * @returns A writable store containing the FilterStoreWrapper
 * @throws Error if called outside of FilterStoreProvider context
 *
 * @example
 * ```svelte
 * <script>
 *   import { useFilterStore } from './providers/filters/useFilterStore.js';
 *
 *   const filterStore = useFilterStore();
 *
 *   function updateOwners(addresses) {
 *     $filterStore = $filterStore?.update(builder =>
 *       builder.setOwners(addresses)
 *     );
 *   }
 *
 *   $: currentFilters = $filterStore?.getFilters();
 * </script>
 * ```
 */
export function useFilterStore(): Readable<RaindexFilterStore> {
	const store = getContext<Readable<RaindexFilterStore>>(FILTER_STORE_CONTEXT);

	if (!store) {
		throw new Error(
			'useFilterStore() must be called within a component wrapped by <FilterStoreProvider>'
		);
	}

	return store;
}
