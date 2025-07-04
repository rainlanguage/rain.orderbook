<script lang="ts">
	import { type RaindexOrder } from '@rainlanguage/orderbook';
	import { prepareHistoricalOrderChartData } from '../../services/historicalOrderCharts';
	import TanstackLightweightChartLine from './TanstackLightweightChartLine.svelte';
	import { createQuery } from '@tanstack/svelte-query';
	import { QKEY_ORDER_TRADES_LIST } from '../../queries/keys';

	export let order: RaindexOrder;
	export let colorTheme;
	export let lightweightChartsTheme;

	$: query = createQuery({
		queryKey: [QKEY_ORDER_TRADES_LIST, order.id],
		queryFn: async () => {
			const data = await order.getTradesList(BigInt(1000), undefined, 1);
			if (data.error) throw new Error(data.error.readableMsg);
			return prepareHistoricalOrderChartData(data.value, $colorTheme);
		}
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
