<script lang="ts">
	import { getOrderTradesList } from '@rainlanguage/orderbook/js_api';
	import { QKEY_ORDER_TRADES_LIST } from '@rainlanguage/ui-components';
	import { createQuery } from '@tanstack/svelte-query';
	import { page } from '$app/stores';
	import TradeDetail from './TradeDetail.svelte';
	const { subgraphUrl } = $page.data.stores
	const { id } = $page.params;

	$: console.log($subgraphUrl)

	// TODO: Going directly to the page, the subgraphUrl is undefined

	$: query = createQuery({
		queryKey: [QKEY_ORDER_TRADES_LIST, id],
		queryFn: () => {
			return getOrderTradesList(
				$subgraphUrl || '',
				id,
				{
					page: 1,
					pageSize: 10
				},
				BigInt(1000),
				undefined
			);
		},
		enabled: !!$subgraphUrl
	});

	$: console.log($query.data)
</script>

{#if $query.data}
	{#each $query.data as trade}
		{trade.id}
		<TradeDetail {trade}  />
	{/each}
{/if}
