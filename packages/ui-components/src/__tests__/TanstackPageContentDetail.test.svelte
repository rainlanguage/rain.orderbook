<script lang="ts">
	import TanstackPageContentDetail from '../lib/components/detail/TanstackPageContentDetail.svelte';

	export let query;
	export let emptyMessage: string;
	export let below: string;
</script>

<TanstackPageContentDetail {query} {emptyMessage}>
	<svelte:fragment slot="top" let:data>
		<span data-testid="top">{data}</span>
	</svelte:fragment>
	<svelte:fragment slot="card" let:data>
		<span data-testid="card">{data}</span>
	</svelte:fragment>
	<svelte:fragment slot="chart" let:data>
		<span data-testid="chart">{data}</span>
	</svelte:fragment>
	<svelte:fragment slot="below">
		<span data-testid="below">{below}</span>
	</svelte:fragment>
</TanstackPageContentDetail>
