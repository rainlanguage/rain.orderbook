<script lang="ts">
	import TanstackAppTable from '../lib/components/TanstackAppTable.svelte';
	export let query;
	export let emptyMessage: string;
	export let title: string;
	export let head: string;
	export let queryKey: string;
	export let enableVirtualization = true;
	export let estimatedRowHeight = 56;
	export let virtualizationOverscan = 8;
</script>

<TanstackAppTable
	{query}
	{emptyMessage}
	rowHoverable
	{queryKey}
	{enableVirtualization}
	{estimatedRowHeight}
	{virtualizationOverscan}
>
	<svelte:fragment slot="title">
		<h2>{title}</h2>
	</svelte:fragment>
	<svelte:fragment slot="head">{head}</svelte:fragment>
	<svelte:fragment slot="bodyRow" let:item>
		{item}
	</svelte:fragment>
</TanstackAppTable>
