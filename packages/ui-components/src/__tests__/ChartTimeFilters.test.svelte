<script>
	import ChartTimeFilters from '../lib/components/charts/ChartTimeFilters.svelte';
	import { writable } from 'svelte/store';

	export let timeDeltaStore = writable(60 * 60 * 24 * 365); // Default to 1 year
</script>

<ChartTimeFilters bind:timeDelta={$timeDeltaStore} />
