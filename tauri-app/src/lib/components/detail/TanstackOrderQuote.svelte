<script lang="ts" generics="T">
  import Refresh from '../icon/Refresh.svelte';

  import type { Order } from '$lib/typeshare/orderDetail';
  import { QKEY_ORDER_QUOTE } from '$lib/queries/keys';
  import { batchOrderQuotes } from '$lib/services/orderQuote';
  import { formatUnits } from 'viem';

  import { createQuery } from '@tanstack/svelte-query';
  import {
    Spinner,
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
  } from 'flowbite-svelte';

  export let id: string;
  export let order: Order;
  let error: string | undefined = undefined;

  const getOrderQuote = async (order: Order) => {
    const data = await batchOrderQuotes([order]);
    return data;
  };

  const refreshQuotes = () => {
    $orderQuoteQuery.refetch();
  };

  $: orderQuoteQuery = createQuery({
    queryKey: [QKEY_ORDER_QUOTE + id],
    queryFn: () => getOrderQuote(order),
    enabled: !!id,
    refetchInterval: 10000,
  });
</script>

<div class="mt-4">
  <div class="mb-4 flex items-center justify-between">
    <h2 class="text-lg font-semibold">Order Quotes</h2>
    <Refresh
      data-testid="refreshButton"
      class="ml-2 h-8 w-5 cursor-pointer text-gray-400 dark:text-gray-400"
      on:click={refreshQuotes}
      spin={$orderQuoteQuery.isLoading || $orderQuoteQuery.isFetching}
    />
  </div>

  <Table divClass="rounded-lg overflow-hidden dark:border-none border">
    <TableHead data-testid="head">
      <TableHeadCell data-testid="orderQuotesPair">Pair</TableHeadCell>
      <TableHeadCell data-testid="orderQuotesMaxOutput">Maximum Output</TableHeadCell>
      <TableHeadCell data-testid="orderQuotesPrice">Price</TableHeadCell>
    </TableHead>

    <TableBody>
      {#if $orderQuoteQuery.isFetching || $orderQuoteQuery.isLoading}
        <TableBodyRow>
          <TableBodyCell colspan="3" class="text-center">
            <Spinner class="h-8 w-8" color="white" data-testid="loadingSpinner" />
          </TableBodyCell>
        </TableBodyRow>
      {:else if $orderQuoteQuery.data && $orderQuoteQuery.data.length > 0}
        {#each $orderQuoteQuery.data as item}
          {#if item.success && item.data}
            <TableBodyRow data-testid="bodyRow">
              <TableBodyCell>{item.pair_name}</TableBodyCell>
              <TableBodyCell>{formatUnits(BigInt(item.data.maxOutput), 18)}</TableBodyCell>
              <TableBodyCell>{formatUnits(BigInt(item.data.ratio), 18)}</TableBodyCell>
            </TableBodyRow>
          {:else if !item.success && item.error}
            <TableBodyRow>
              <TableBodyCell>{item.pair_name}</TableBodyCell>
              <TableBodyCell colspan="2" class="text-center text-red-500 dark:text-red-400">
                {'Error fetching pair quote:'} <br />
                {item.error}
              </TableBodyCell>
            </TableBodyRow>
          {/if}
        {/each}
      {:else if error}
        <TableBodyRow>
          <TableBodyCell colspan="3" class="text-center text-red-500 dark:text-red-400">
            {'Error fetching pair quote:'} <br />
            {error}
          </TableBodyCell>
        </TableBodyRow>
      {:else}
        <TableBodyRow>
          <TableBodyCell colspan="3" class="text-center text-gray-900 dark:text-white">
            {'Max output and price not found'}
          </TableBodyCell>
        </TableBodyRow>
      {/if}
    </TableBody>
  </Table>
</div>
