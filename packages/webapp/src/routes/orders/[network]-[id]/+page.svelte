<script lang="ts">
	import { page } from '$app/stores';
	import {
		OrderTradesChart,
		OrderTradesListTable,
		colorTheme,
		lightweightChartsTheme,
		TanstackOrderQuote,
		QKEY_ORDER,
		OrderVaultsVolTable,
		CodeMirrorRainlang,
		lightCodeMirrorTheme
	} from '@rainlanguage/ui-components';
	import { getOrder } from '@rainlanguage/orderbook/js_api';
	import { createQuery } from '@tanstack/svelte-query';
	const { id, network } = $page.params;
	const { settings } = $page.data.stores;
	const subgraphUrl = $settings.subgraphs[network];
	const rpcUrl = $settings.networks[network]?.rpc;

	$: orderDetailQuery = createQuery({
		queryKey: [id, QKEY_ORDER + id],
		queryFn: () => getOrder(subgraphUrl, id || ''),
		enabled: !!subgraphUrl && !!id
	});

	$: order = $orderDetailQuery.data;
	$: console.log('ORDER', order, 'RAINLANG', order.rainlang);
</script>

<h1>Order Trades</h1>
{#if $orderDetailQuery.data}
	<CodeMirrorRainlang
		disabled={true}
		value={order.rainlang}
		codeMirrorTheme={lightCodeMirrorTheme}
	/>

	<TanstackOrderQuote {id} {order} {rpcUrl} />
	<OrderTradesChart {id} {subgraphUrl} {colorTheme} {lightweightChartsTheme} />
	<OrderTradesListTable {id} {subgraphUrl} {rpcUrl} />
	<OrderVaultsVolTable {id} {subgraphUrl} />
{/if}
