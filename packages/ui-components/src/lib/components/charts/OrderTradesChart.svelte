<script lang="ts">
	import { getOrderTradesList } from '@rainlanguage/orderbook/js_api';
	import { prepareHistoricalOrderChartData } from '../../services/historicalOrderCharts';
	import TanstackLightweightChartLine from './TanstackLightweightChartLine.svelte';
	import { createQuery } from '@tanstack/svelte-query';
	import { QKEY_ORDER_TRADES_LIST } from '../../queries/keys';
	import { bigintToFloat } from '$lib/utils/number';

	export let id: string;
	export let subgraphUrl: string;
	export let colorTheme;
	export let lightweightChartsTheme;

	$: query = createQuery({
		queryKey: [QKEY_ORDER_TRADES_LIST, id],
		queryFn: async () => {
			const data = await getOrderTradesList(
				subgraphUrl || '',
				id,
				{
					page: 1,
					pageSize: 10
				},
				BigInt(1000),
				undefined
			);
			return prepareHistoricalOrderChartData(data, $colorTheme);
		},
		enabled: !!subgraphUrl
	});

	const Chart = TanstackLightweightChartLine;
</script>

{#if $query.data}
	<Chart
		title="Trades"
		{query}
		timeTransform={(d) => d.time}
		valueTransform={(d) => bigintToFloat(BigInt(d.outputAmount), 18)}
		emptyMessage="No trades found"
		{lightweightChartsTheme}
	/>
{/if}
