<script>
	import TableTimeFilters from '../lib/components/charts/TableTimeFilters.svelte';
	import { writable } from 'svelte/store';

	export let startTimeStore = writable();
	export let endTimeStore = writable();
</script>

<TableTimeFilters bind:startTimestamp={$startTimeStore} bind:endTimestamp={$endTimeStore} />
