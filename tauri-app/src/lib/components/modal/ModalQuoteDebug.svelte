<script lang="ts">
  import { debugOrderQuote } from '$lib/queries/orderQuote';
  import { queryClient } from '$lib/queries/queryClient';
  import type { Order } from '$lib/typeshare/orderDetail';
  import { createQuery } from '@tanstack/svelte-query';
  import {
    Alert,
    Modal,
    Spinner,
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
  } from 'flowbite-svelte';
  import { formatEther, hexToBigInt, type Hex } from 'viem';
  import Refresh from '../icon/Refresh.svelte';

  export let open: boolean;
  export let order: Order;
  export let inputIOIndex: number;
  export let outputIOIndex: number;
  export let pair: string;
  export let orderbook: Hex;
  export let rpcUrl: string;

  $: debugQuery = createQuery(
    {
      queryKey: [order + rpcUrl],
      queryFn: () => {
        return debugOrderQuote(order, inputIOIndex, outputIOIndex, orderbook, rpcUrl);
      },
      retry: 0,
    },
    queryClient,
  );
</script>

<Modal title={`Debugging quote for pair ${pair}`} bind:open outsideclose size="lg">
  <div class="flex w-full justify-end">
    <Refresh
      data-testid="refreshButton"
      class="ml-2 h-8 w-5 cursor-pointer text-gray-400 dark:text-gray-400"
      on:click={() => $debugQuery.refetch()}
      spin={$debugQuery.isLoading || $debugQuery.isFetching}
    />
  </div>
  {#if $debugQuery.isLoading}
    <div class="flex items-center gap-x-2">
      <Spinner size="4" />
      <span data-testid="modal-quote-debug-loading-message">Getting quote stack...</span>
    </div>
  {/if}
  {#if $debugQuery.isError}
    <Alert data-testid="modal-quote-debug-error" color="red">{$debugQuery.error}</Alert>
  {/if}
  {#if $debugQuery.data}
    <Table divClass="cursor-pointer rounded-lg overflow-hidden dark:border-none border">
      <TableHead>
        <TableHeadCell>Stack item</TableHeadCell>
        <TableHeadCell>Value</TableHeadCell>
        <TableHeadCell>Hex</TableHeadCell>
      </TableHead>
      <TableBody>
        {#each $debugQuery.data as value, i}
          <TableBodyRow>
            <TableBodyCell data-testid="modal-quote-debug-stack">{i}</TableBodyCell>
            <TableBodyCell data-testid="modal-quote-debug-value"
              >{formatEther(hexToBigInt(value))}</TableBodyCell
            >
            <TableBodyCell data-testid="modal-quote-debug-value-hex">{value}</TableBodyCell>
          </TableBodyRow>
        {/each}
      </TableBody>
    </Table>
  {/if}
  <div class="flex flex-col gap-y-2 text-sm">
    <span data-testid="modal-quote-debug-rpc-url">RPC: {rpcUrl}</span>
  </div>
</Modal>
