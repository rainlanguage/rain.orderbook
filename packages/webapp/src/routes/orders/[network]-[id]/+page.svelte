<script lang="ts">
	import { page } from '$app/stores';
	import {
		OrderTradesChart,
		colorTheme,
		lightweightChartsTheme
	} from '@rainlanguage/ui-components';
	const { id, network } = $page.params;
	const { settings } = $page.data.stores;
	$: console.log(settings);
	const subgraphUrl = $settings.subgraphs[network];
</script>

<h1>Order Trades</h1>
<OrderTradesChart {id} {subgraphUrl} {colorTheme} {lightweightChartsTheme} />
