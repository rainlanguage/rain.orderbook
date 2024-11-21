<script lang="ts" generics="T">
  import { orderbookAddress, rpcUrl } from '$lib/stores/settings';
  import { handleQuoteDebugModal } from '$lib/services/modal';
  import { Refresh } from '@rainlanguage/ui-components';
  import type { Order } from '$lib/typeshare/subgraphTypes';
  import { QKEY_ORDER_QUOTE } from '@rainlanguage/ui-components';
  import { batchOrderQuotes } from '$lib/queries/orderQuote';
  import { formatUnits, hexToNumber, isHex } from 'viem';
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
  import EditableSpan from '../EditableSpan.svelte';

  export let id: string;
  export let order: Order;

  let enabled = true;

  const getOrderQuote = async (order: Order, blockNumber?: number) => {
    const data = await batchOrderQuotes([order], blockNumber);
    return data;
  };

  const refreshQuotes = () => {
    $orderQuoteQuery.refetch();
  };

  $: orderQuoteQuery = createQuery({
    queryKey: [QKEY_ORDER_QUOTE + id],
    queryFn: () => getOrderQuote(order, blockNumber),
    enabled: !!id && enabled,
    refetchInterval: 10000,
  });

  let blockNumber: number | undefined;
</script>

<div class="mt-4">
  <div class="mb-4 flex items-center justify-between">
    <h2 class="text-lg font-semibold">Order Quotes</h2>
    <div class="flex items-center gap-x-1">
      {#if $orderQuoteQuery.data && isHex($orderQuoteQuery.data[0].block_number)}
        <EditableSpan
          displayValue={blockNumber?.toString() ||
            hexToNumber($orderQuoteQuery.data[0].block_number).toString()}
          on:focus={() => {
            enabled = false;
          }}
          on:blur={({ detail: { value } }) => {
            blockNumber = parseInt(value);
            refreshQuotes();
          }}
        />
      {/if}
      <span></span>
      <Refresh
        data-testid="refreshButton"
        class="h-8 w-5 cursor-pointer text-gray-400 dark:text-gray-400"
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
          blockNumber = undefined;
          refreshQuotes();
        }}
        class={`ml-2 h-8 w-3 cursor-pointer text-gray-400 dark:text-gray-400 ${enabled ? 'hidden' : ''}`}
      />
    </div>
  </div>

  <Table divClass="rounded-lg overflow-hidden dark:border-none border">
    <TableHead data-testid="head">
      <TableHeadCell class="w-[80px]" data-testid="orderQuotesPair">Pair</TableHeadCell>
      <TableHeadCell class="w-1/4" data-testid="orderQuotesMaxOutput">Maximum Output</TableHeadCell>
      <TableHeadCell class="w-1/4" data-testid="orderQuotesPrice">Price</TableHeadCell>
      <TableHeadCell data-testid="orderQuotesPrice">Maximum Input</TableHeadCell>
      <TableHeadCell class="w-[50px]" />
    </TableHead>

    <TableBody>
      {#if $orderQuoteQuery.data && $orderQuoteQuery.data.length > 0}
        {#each $orderQuoteQuery.data as item}
          {#if item.success && item.data}
            <TableBodyRow data-testid="bodyRow">
              <TableBodyCell>{item.pair.pair_name}</TableBodyCell>
              <TableBodyCell>{formatUnits(BigInt(item.data.maxOutput), 18)}</TableBodyCell>
              <TableBodyCell
                >{formatUnits(BigInt(item.data.ratio), 18)}
                <span class="text-gray-400"
                  >({BigInt(item.data.ratio) > 0n
                    ? formatUnits(10n ** 36n / BigInt(item.data.ratio), 18)
                    : '0'})</span
                ></TableBodyCell
              >
              <TableBodyCell
                >{formatUnits(
                  BigInt(item.data.maxOutput) * BigInt(item.data.ratio),
                  36,
                )}</TableBodyCell
              >
              <TableBodyCell>
                <button
                  on:click={() =>
                    handleQuoteDebugModal(
                      order,
                      $rpcUrl || '',
                      $orderbookAddress || '',
                      item.pair.input_index,
                      item.pair.output_index,
                      item.pair.pair_name,
                      parseInt($orderQuoteQuery.data[0].block_number),
                    )}
                >
                  <BugOutline size="sm" color="grey" />
                </button>
              </TableBodyCell>
            </TableBodyRow>
          {:else if !item.success && item.error}
            <TableBodyRow>
              <TableBodyCell>{item.pair.pair_name}</TableBodyCell>
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
              <TableBodyCell>
                <button
                  on:click={() =>
                    handleQuoteDebugModal(
                      order,
                      $rpcUrl || '',
                      $orderbookAddress || '',
                      item.pair.input_index,
                      item.pair.output_index,
                      item.pair.pair_name,
                      parseInt($orderQuoteQuery.data[0].block_number),
                    )}
                >
                  <BugOutline size="sm" color="grey" />
                </button>
              </TableBodyCell>
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
