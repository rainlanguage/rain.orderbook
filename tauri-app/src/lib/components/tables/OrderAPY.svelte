<script lang="ts">
  import { createInfiniteQuery } from '@tanstack/svelte-query';
  import TanstackAppTable from './TanstackAppTable.svelte';
  import { QKEY_ORDER_APY } from '$lib/queries/keys';
  import { getOrderApy } from '$lib/queries/orderTradesList';
  import { subgraphUrl } from '$lib/stores/settings';
  import { TableBodyCell, TableHeadCell } from 'flowbite-svelte';
  import ApyTimeFilters from '../charts/APYTimeFilters.svelte';

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
</script>

<TanstackAppTable query={orderApy} emptyMessage="No Order found" rowHoverable={false}>
  <svelte:fragment slot="timeFilter">
    <ApyTimeFilters bind:startTimestamp bind:endTimestamp />
  </svelte:fragment>
  <svelte:fragment slot="head">
    <TableHeadCell padding="p-4">APY</TableHeadCell>
  </svelte:fragment>

  <svelte:fragment slot="bodyRow" let:item>
    <TableBodyCell tdClass="break-all px-4 py-2" data-testid="apy">
      {item.apy?.apy ?? 0} % {item.apy?.token?.symbol ? 'in ' + item.apy.token.symbol : ''}
    </TableBodyCell>
  </svelte:fragment>
</TanstackAppTable>
