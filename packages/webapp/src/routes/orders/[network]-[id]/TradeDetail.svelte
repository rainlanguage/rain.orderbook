<script lang="ts">
	import { createQuery } from '@tanstack/svelte-query';
	import { getOrderTradeDetail } from '@rainlanguage/orderbook/js_api';
	import { page } from '$app/stores';
	export let trade;
	const { subgraphUrl } = $page.data.stores;

	$: query = createQuery({
		queryKey: [trade],
		queryFn: () => {
			return getOrderTradeDetail($subgraphUrl || '', trade.id);
		},
		enabled: !!$subgraphUrl
	});
</script>

<div class="border border-green-500">
	<h1>Trade Detail</h1>
	{#if $query.data}
		{$query.data.timestamp}
	{/if}
</div>
