import { getContext } from 'svelte';
import { FILTER_STORE_CONTEXT, type FilterStoreContext } from './FilterStoreProvider.svelte';

/**
 * Hook for accessing the filter store from any Svelte component.
 * Must be used within a component that is wrapped by FilterStoreProvider.
 *
 * @returns A FilterStoreContext containing the filter store and current vaults filters.
 * @throws Error if called outside of FilterStoreProvider context
 *
 * @example
 * ```svelte
 * <script>
 *   import { useFilterStore } from '@rainlanguage/ui-components';
 *
 *   const { filterStore, currentVaultsFilters } = useFilterStore();
 *
 *   const updateOwners = (addresses) => {
 *     $filterStore?.updateVaults((builder) => builder.setOwners(addresses));
 *   };
 *
 *   $: console.log('Current vault filters', $currentVaultsFilters);
 * </script>
 * ```
 */
export function useFilterStore(): FilterStoreContext {
	const store = getContext<FilterStoreContext>(FILTER_STORE_CONTEXT);

	if (!store) {
		throw new Error(
			'useFilterStore() must be called within a component wrapped by <FilterStoreProvider>'
		);
	}

	return store;
}
