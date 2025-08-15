<script lang="ts" context="module">
	import type { GetVaultsFilters } from '@rainlanguage/orderbook';
	import type { Readable } from 'svelte/store';
	/**
	 * Svelte context key for the filter store
	 */
	export const FILTER_STORE_CONTEXT = 'filterStore';

	export const DEFAULT_VAULT_FILTERS: GetVaultsFilters = {
		owners: [],
		hideZeroBalance: false,
		tokens: undefined,
		chainIds: undefined
	};

	export type FilterStoreContext = {
		filterStore: Readable<RaindexFilterStore>;
		currentVaultsFilters: Readable<GetVaultsFilters>;
	};
</script>

<script lang="ts">
	import { setContext, onMount } from 'svelte';
	import { writable, type Writable } from 'svelte/store';
	import { RaindexFilterStore } from './RaindexFilterStore';

	/**
	 * Create a writable store containing the FilterStoreWrapper
	 */
	const filterStore: Writable<RaindexFilterStore | null> = writable(null);
	const currentVaultsFilters = writable(DEFAULT_VAULT_FILTERS);

	/**
	 * Initialize the filter store on component mount
	 */
	onMount(() => {
		const wrapper = new RaindexFilterStore();
		filterStore.set(wrapper);

		wrapper.subscribe((store) => {
			if (store) {
				currentVaultsFilters.set({
					...DEFAULT_VAULT_FILTERS,
					...store.getVaultsFilters()
				});
			} else {
				currentVaultsFilters.set(DEFAULT_VAULT_FILTERS);
			}
		});
	});

	/**
	 * Set the context so child components can access the store
	 */
	setContext(FILTER_STORE_CONTEXT, {
		filterStore,
		currentVaultsFilters
	});
</script>

<!--
  This provider initializes and provides the FilterStoreWrapper to all child components.

  Usage:
  ```svelte
  <FilterStoreProvider>
    <YourApp />
  </FilterStoreProvider>
  ```

  In child components:
  ```svelte
  <script>
    import { getContext } from 'svelte';
    import { FILTER_STORE_CONTEXT } from './providers/filters/FilterStoreProvider.svelte';

    const filterStore = getContext(FILTER_STORE_CONTEXT);

    // Update filters
    $filterStore = $filterStore?.update(builder =>
      builder.setOwners([address]).setHideZeroBalance(true)
    );
  </script>
  ```
-->
<slot />
