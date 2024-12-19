<script lang="ts">
	import { createInfiniteQuery } from '@tanstack/svelte-query';
	import TanstackAppTable from '../TanstackAppTable.svelte';
	import { QKEY_ORDER_APY } from '../../queries/keys';
	import { getOrderPerformance, type OrderPerformance } from '@rainlanguage/orderbook/js_api';
	import { TableBodyCell, TableHeadCell } from 'flowbite-svelte';
	import ApyTimeFilters from '../charts/APYTimeFilters.svelte';
	import { bigintStringToPercentage } from '$lib/utils/number';

	export let id: string;
	export let subgraphUrl: string;

	let startTimestamp: number | undefined;
	let endTimestamp: number | undefined;

	$: queryStartTime = startTimestamp ? BigInt(startTimestamp) : undefined;
	$: queryEndTime = endTimestamp ? BigInt(endTimestamp) : undefined;

	$: orderPerformance = createInfiniteQuery({
		queryKey: [id, QKEY_ORDER_APY + id],
		queryFn: async () => {
			return [
				(await getOrderPerformance(
					subgraphUrl || '',
					id,
					queryStartTime,
					queryEndTime
				)) as OrderPerformance
			];
		},
		initialPageParam: 0,
		getNextPageParam: () => undefined,
		enabled: !!subgraphUrl
	});
</script>

<TanstackAppTable query={orderPerformance} emptyMessage="APY Unavailable" rowHoverable={false}>
	<svelte:fragment slot="timeFilter">
		<ApyTimeFilters bind:startTimestamp bind:endTimestamp />
	</svelte:fragment>
	<svelte:fragment slot="head">
		<TableHeadCell padding="p-4">APY</TableHeadCell>
	</svelte:fragment>

	<svelte:fragment slot="bodyRow" let:item>
		<TableBodyCell tdClass="break-all px-4 py-2" data-testid="apy-field">
			{item.denominatedPerformance
				? (item.denominatedPerformance.apyIsNeg ? '-' : '') +
					bigintStringToPercentage(item.denominatedPerformance.apy, 18, 5) +
					'% in ' +
					(item.denominatedPerformance.token.symbol ??
						item.denominatedPerformance.token.name ??
						item.denominatedPerformance.token.address)
				: 'Unavailable APY'}
		</TableBodyCell>
	</svelte:fragment>
</TanstackAppTable>
