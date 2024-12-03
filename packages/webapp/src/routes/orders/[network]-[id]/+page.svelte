<script lang="ts">
	import { page } from '$app/stores';
	import {
		OrderTradesChart,
		OrderTradesListTable,
		colorTheme,
		lightweightChartsTheme,
		TanstackOrderQuote,
		QKEY_ORDER
	} from '@rainlanguage/ui-components';
	import { getOrder } from '@rainlanguage/orderbook/js_api';
	import { createQuery } from '@tanstack/svelte-query';
	const { id, network } = $page.params;
	const { settings } = $page.data.stores;
	const subgraphUrl = $settings.subgraphs[network];
	const rpcUrl = $settings.networks[network]?.rpc;
	$: console.log(rpcUrl);
	$: orderDetailQuery = createQuery({
		queryKey: [id, QKEY_ORDER + id],
		queryFn: () => getOrder(subgraphUrl, id || ''),
		enabled: !!subgraphUrl && !!id
	});

	$: order = $orderDetailQuery.data;
	$: console.log(order);
</script>

<h1>Order Trades</h1>
{#if $orderDetailQuery.data}
	Hi!
	<TanstackOrderQuote {id} {order} />
	<OrderTradesChart {id} {subgraphUrl} {colorTheme} {lightweightChartsTheme} />
	<OrderTradesListTable {id} {subgraphUrl} {rpcUrl} />
{/if}
