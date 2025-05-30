<script lang="ts">
  import { queryClient } from '$lib/queries/queryClient';
  import { tradeDebug } from '$lib/queries/tradeDebug';
  import { createQuery } from '@tanstack/svelte-query';
  import { Alert, Modal, Spinner } from 'flowbite-svelte';
  import EvalResultsTable from '../debug/EvalResultsTable.svelte';

  export let open: boolean;
  export let txHash: string;
  export let rpcUrls: string[];

  $: debugQuery = createQuery(
    {
      queryKey: [txHash + rpcUrls.join(',')],
      queryFn: () => {
        return tradeDebug(txHash, rpcUrls);
      },
      retry: 0,
    },
    queryClient,
  );
</script>

<Modal title="Debug trade" bind:open outsideclose size="lg">
  <div class="flex flex-col gap-y-2 text-sm">
    <span data-testid="modal-trade-debug-tx-hash">Trade transaction: {txHash}</span>
    <span data-testid="modal-trade-debug-rpc-url">RPCs: {rpcUrls.join(', ')}</span>
  </div>
  {#if $debugQuery.isLoading}
    <div data-testid="modal-trade-debug-loading-message" class="flex items-center gap-x-2">
      <Spinner size="4" />
      <span>Replaying trade... this can take a while.</span>
    </div>
  {/if}
  {#if $debugQuery.isError}
    <Alert data-testid="modal-trade-debug-error" color="red">{$debugQuery.error}</Alert>
  {/if}
  {#if $debugQuery.data}
    <EvalResultsTable table={$debugQuery.data} />
  {/if}
</Modal>
