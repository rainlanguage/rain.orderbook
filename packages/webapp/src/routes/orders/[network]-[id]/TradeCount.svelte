<script lang="ts">
	import { createQuery } from '@tanstack/svelte-query';
	import { getOrderTradesCount } from '@rainlanguage/orderbook/js_api';
	import { page } from '$app/stores';
	export let orderId;
	const { subgraphUrl } = $page.data.stores;

	$: query = createQuery({
		queryKey: [orderId],
		queryFn: () => {
			return getOrderTradesCount($subgraphUrl || '', orderId, undefined, undefined);
		},
		enabled: !!$subgraphUrl
	});
</script>

<div class="border border-red-500">
	<h1>Trade Count</h1>
	{#if $query.data}
		{$query.data}
	{/if}
</div>
