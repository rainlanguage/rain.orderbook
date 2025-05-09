<script lang="ts">
	import { getOrderTradesList } from '@rainlanguage/orderbook';
	import { prepareHistoricalOrderChartData } from '../../services/historicalOrderCharts';
	import TanstackLightweightChartLine from './TanstackLightweightChartLine.svelte';
	import { createQuery } from '@tanstack/svelte-query';
	import { QKEY_ORDER_TRADES_LIST } from '../../queries/keys';

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
			if (data.error) throw new Error(data.error.msg);
			return prepareHistoricalOrderChartData(data.value, $colorTheme);
		},
		enabled: !!subgraphUrl
	});
</script>

<TanstackLightweightChartLine
	title="Trades"
	{query}
	timeTransform={(d) => d.time}
	valueTransform={(d) => d.value}
	emptyMessage="No trades found"
	{lightweightChartsTheme}
/>
