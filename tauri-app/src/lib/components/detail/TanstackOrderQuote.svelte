<script lang="ts" generics="T">
  import { orderbookAddress, rpcUrl } from '$lib/stores/settings';

  import { handleQuoteDebugModal } from '$lib/services/modal';

  import Refresh from '../icon/Refresh.svelte';
  import type { Order } from '$lib/typeshare/orderDetail';
  import { QKEY_ORDER_QUOTE } from '$lib/queries/keys';
  import { batchOrderQuotes } from '$lib/queries/orderQuote';
  import { formatUnits } from 'viem';
  import { createQuery } from '@tanstack/svelte-query';
  import {
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
    Tooltip,
  } from 'flowbite-svelte';
  import { BugOutline, PauseSolid, PlaySolid } from 'flowbite-svelte-icons';

  export let id: string;
  export let order: Order;

  let enabled = true;

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
    enabled: !!id && enabled,
    refetchInterval: 10000,
  });
</script>

<div class="mt-4">
  <div class="mb-4 flex items-center justify-between">
    <h2 class="text-lg font-semibold">Order Quotes</h2>
    <div class="flex gap-x-2">
      <Refresh
        data-testid="refreshButton"
        class="ml-2 h-8 w-5 cursor-pointer text-gray-400 dark:text-gray-400"
        on:click={refreshQuotes}
        spin={$orderQuoteQuery.isLoading || $orderQuoteQuery.isFetching}
      />
      <PauseSolid
        class={`ml-2 h-8 w-3 cursor-pointer text-gray-400 dark:text-gray-400 ${!enabled ? 'hidden' : ''}`}
        on:click={() => {
          enabled = false;
        }}
      />
      <PlaySolid
        on:click={() => {
          enabled = true;
          refreshQuotes();
        }}
        class={`ml-2 h-8 w-3 cursor-pointer text-gray-400 dark:text-gray-400 ${enabled ? 'hidden' : ''}`}
      />
    </div>
  </div>

  <Table divClass="rounded-lg overflow-hidden dark:border-none border">
    <TableHead data-testid="head">
      <TableHeadCell data-testid="orderQuotesPair">Pair</TableHeadCell>
      <TableHeadCell data-testid="orderQuotesMaxOutput">Maximum Output</TableHeadCell>
      <TableHeadCell data-testid="orderQuotesPrice">Price</TableHeadCell>
      <TableHeadCell />
    </TableHead>

    <TableBody>
      {#if $orderQuoteQuery.data && $orderQuoteQuery.data.length > 0}
        {#each $orderQuoteQuery.data as item}
          {#if item.success && item.data}
            <TableBodyRow data-testid="bodyRow">
              <TableBodyCell>{item.pair_name}</TableBodyCell>
              <TableBodyCell>{formatUnits(BigInt(item.data.maxOutput), 18)}</TableBodyCell>
              <TableBodyCell>{formatUnits(BigInt(item.data.ratio), 18)}</TableBodyCell>
              <TableBodyCell>
                <button
                  on:click={() =>
                    handleQuoteDebugModal(
                      order,
                      $rpcUrl || '',
                      $orderbookAddress || '',
                      item.inputIOIndex,
                      item.outputIOIndex,
                      item.pair_name,
                    )}
                >
                  <BugOutline />
                </button>
              </TableBodyCell>
            </TableBodyRow>
          {:else if !item.success && item.error}
            <TableBodyRow>
              <TableBodyCell>{item.pair_name}</TableBodyCell>
              <TableBodyCell colspan="2" class="flex flex-col justify-start text-gray-400">
                <Tooltip triggeredBy="#quote-error">
                  {item.error}
                </Tooltip>
                <div
                  id="quote-error"
                  class="overflow-x cursor-pointer self-start border-dotted border-red-500"
                >
                  Error fetching quote
                </div>
              </TableBodyCell>
              <TableBodyCell />
              <TableBodyCell />
            </TableBodyRow>
          {/if}
        {/each}
      {:else if $orderQuoteQuery.isError}
        <TableBodyRow>
          <TableBodyCell colspan="3" class="text-center text-red-500 dark:text-red-400">
            {'Error fetching quotes:'} <br />
            {$orderQuoteQuery.error}
          </TableBodyCell>
        </TableBodyRow>
      {/if}
    </TableBody>
  </Table>
</div>
