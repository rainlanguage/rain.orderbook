<script lang="ts">
  import { debugOrderQuote } from '$lib/queries/orderQuote';
  import { queryClient } from '$lib/queries/queryClient';
  import type { Order } from '@rainlanguage/orderbook/js_api';
  import { createQuery } from '@tanstack/svelte-query';
  import { Alert, Modal } from 'flowbite-svelte';
  import { type Hex } from 'viem';
  import { Refresh, EvalResultsTable } from '@rainlanguage/ui-components';
  import { fade } from 'svelte/transition';

  export let open: boolean;
  export let order: Order;
  export let inputIOIndex: number;
  export let outputIOIndex: number;
  export let pair: string;
  export let orderbook: Hex;
  export let rpcUrl: string;
  export let blockNumber: number | undefined;

  $: debugQuery = createQuery(
    {
      queryKey: [order + rpcUrl + pair + blockNumber],
      queryFn: () => {
        return debugOrderQuote(order, inputIOIndex, outputIOIndex, orderbook, rpcUrl, blockNumber);
      },
      retry: 0,
      refetchOnWindowFocus: false,
      refetchInterval: false,
      refetchOnMount: true,
    },
    queryClient,
  );
</script>

<Modal title={`Debugging quote for pair ${pair}`} bind:open outsideclose size="lg">
  <div class="flex items-center">
    {#if $debugQuery.data}
      <div class="flex flex-col text-sm">
        <span class="whitespace-nowrap" data-testid="modal-quote-debug-rpc-url">RPC: {rpcUrl}</span>
        <span class="whitespace-nowrap" data-testid="modal-quote-debug-block-number"
          >Block: {blockNumber}</span
        >
      </div>
    {/if}
    <div class="flex w-full items-center justify-end">
      {#if $debugQuery.isLoading || $debugQuery.isFetching}
        <span class="text-sm" transition:fade data-testid="modal-quote-debug-loading-message"
          >Getting quote stack...</span
        >
      {/if}
      <Refresh
        data-testid="refreshButton"
        class="ml-2 h-8 w-5 cursor-pointer text-gray-400 dark:text-gray-400"
        on:click={() => $debugQuery.refetch()}
        spin={$debugQuery.isLoading || $debugQuery.isFetching}
      />
    </div>
  </div>
  {#if $debugQuery.data}
    {#if !!$debugQuery.data[1]}
      <Alert data-testid="modal-quote-debug-error-partial" color="red">{$debugQuery.data[1]}</Alert>
    {/if}
    <EvalResultsTable table={$debugQuery.data[0]} />
  {/if}
  <div class="flex flex-col gap-y-2 text-sm"></div>
</Modal>
