<script lang="ts">
	import { page } from '$app/stores';
	import {
		OrderTradesChart,
		OrderTradesListTable,
		colorTheme,
		lightweightChartsTheme,
		TanstackOrderQuote
	} from '@rainlanguage/ui-components';
	const { id, network } = $page.params;
	const { settings } = $page.data.stores;
	const subgraphUrl = $settings.subgraphs[network];
	const rpcUrl = $settings.networks[network]?.rpc;
</script>

<h1>Order Trades</h1>
<TanstackOrderQuote {id} {colorTheme} {lightweightChartsTheme} />
<OrderTradesChart {id} {subgraphUrl} {colorTheme} {lightweightChartsTheme} />
<OrderTradesListTable {id} {subgraphUrl} {rpcUrl} />
