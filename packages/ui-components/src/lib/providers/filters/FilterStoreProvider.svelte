<script lang="ts" context="module">
	/**
	 * Svelte context key for the filter store
	 */
	export const FILTER_STORE_CONTEXT = 'filterStore';
</script>

<script lang="ts">
	import { setContext, onMount } from 'svelte';
	import { writable, type Writable } from 'svelte/store';
	import { RaindexFilterStore } from './RaindexFilterStore';

	/**
	 * Create a writable store containing the FilterStoreWrapper
	 */
	const filterStore: Writable<RaindexFilterStore | null> = writable(null);

	/**
	 * Initialize the filter store on component mount
	 */
	onMount(() => {
		const wrapper = new RaindexFilterStore();
		filterStore.set(wrapper);
	});

	/**
	 * Set the context so child components can access the store
	 */
	setContext(FILTER_STORE_CONTEXT, filterStore);
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
