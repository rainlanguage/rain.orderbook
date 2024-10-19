<script lang="ts">
  import { createInfiniteQuery } from '@tanstack/svelte-query';
  import TanstackAppTable from './TanstackAppTable.svelte';
  import { QKEY_ORDER_APY } from '$lib/queries/keys';
  import { getOrderApy } from '$lib/queries/orderTradesList';
  import { subgraphUrl } from '$lib/stores/settings';
  import { TableBodyCell, TableHeadCell } from 'flowbite-svelte';
  import ApyTimeFilters from '../charts/APYTimeFilters.svelte';
  import { formatUnits } from 'viem';

  export let id: string;

  let startTimestamp: number | undefined;
  let endTimestamp: number | undefined;

  $: orderApy = createInfiniteQuery({
    queryKey: [id, QKEY_ORDER_APY + id],
    queryFn: () => getOrderApy(id, $subgraphUrl || '', startTimestamp, endTimestamp),
    initialPageParam: 0,
    getNextPageParam: () => undefined,
    enabled: !!$subgraphUrl,
  });

  function formatApyToPercentage(value: string): string {
    let valueString = formatUnits(BigInt(value) * 100n, 18);
    const index = valueString.indexOf('.');
    if (index > -1) {
      // 5 point decimals to show on UI
      valueString = valueString.substring(0, index + 6);
    }
    return valueString;
  }
</script>

<TanstackAppTable query={orderApy} emptyMessage="APY Unavailable" rowHoverable={false}>
  <svelte:fragment slot="timeFilter">
    <ApyTimeFilters bind:startTimestamp bind:endTimestamp />
  </svelte:fragment>
  <svelte:fragment slot="head">
    <TableHeadCell padding="p-4">APY</TableHeadCell>
  </svelte:fragment>

  <svelte:fragment slot="bodyRow" let:item>
    <TableBodyCell tdClass="break-all px-4 py-2" data-testid="apy-field">
      {item.denominatedApy
        ? formatApyToPercentage(item.denominatedApy.apy) +
          '% in ' +
          (item.denominatedApy.token.symbol ??
            item.denominatedApy.token.name ??
            item.denominatedApy.token.address)
        : 'Unavailable APY'}
    </TableBodyCell>
  </svelte:fragment>
</TanstackAppTable>
