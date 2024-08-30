<script lang="ts">
  import { debugOrderQuote } from '$lib/queries/orderQuote';
  import { queryClient } from '$lib/queries/queryClient';
  import type { Order } from '$lib/typeshare/orderDetail';
  import { createQuery } from '@tanstack/svelte-query';
  import { Alert, Modal, Spinner } from 'flowbite-svelte';
  import { type Hex } from 'viem';
  import Refresh from '../icon/Refresh.svelte';
  import EvalResultsTable from '../debug/EvalResultsTable.svelte';

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
    <EvalResultsTable table={$debugQuery.data} />
  {/if}
  <div class="flex flex-col gap-y-2 text-sm">
    <span data-testid="modal-quote-debug-rpc-url">RPC: {rpcUrl}</span>
  </div>
</Modal>
