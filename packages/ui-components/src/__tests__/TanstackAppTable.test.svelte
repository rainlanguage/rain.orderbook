<script lang="ts">
	import TanstackAppTable from '../lib/components/TanstackAppTable.svelte';
	export let query;
	export let emptyMessage: string;
	export let title: string;
	export let head: string;
	export let queryKey: string;
	export let virtualization: {
		enabled?: boolean;
		estimatedRowHeight?: number;
		overscan?: number;
	} = {};
</script>

<TanstackAppTable {query} {emptyMessage} rowHoverable {queryKey} {virtualization}>
	<svelte:fragment slot="title">
		<h2>{title}</h2>
	</svelte:fragment>
	<svelte:fragment slot="head">{head}</svelte:fragment>
	<svelte:fragment slot="bodyRow" let:item>
		{item}
	</svelte:fragment>
</TanstackAppTable>
