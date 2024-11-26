<script>
	import ChartTimeFilters from './ChartTimeFilters.svelte';
	import { writable } from 'svelte/store';

	export let timeDeltaStore = writable(60 * 60 * 24 * 365); // Default to 1 year
</script>

<ChartTimeFilters bind:timeDelta={$timeDeltaStore} />
