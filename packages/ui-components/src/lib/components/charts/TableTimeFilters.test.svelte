<script>
	import TableTimeFilters from './TableTimeFilters.svelte';
	import { writable } from 'svelte/store';

	export let startTimeStore = writable();
	export let endTimeStore = writable();
</script>

<TableTimeFilters bind:startTimestamp={$startTimeStore} bind:endTimestamp={$endTimeStore} />
