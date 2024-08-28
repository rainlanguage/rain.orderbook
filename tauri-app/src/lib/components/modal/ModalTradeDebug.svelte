<script lang="ts">
  import { queryClient } from '$lib/queries/queryClient';
  import { tradeDebug } from '$lib/queries/tradeDebug';
  import { createQuery } from '@tanstack/svelte-query';
  import { Modal } from 'flowbite-svelte';

  export let open: boolean;
  export let txHash: string;
  export let rpcUrl: string;

  $: debugQuery = createQuery(
    {
      queryKey: [txHash + rpcUrl],
      queryFn: () => {
        return tradeDebug(txHash, rpcUrl);
      },
      retry: 0,
    },
    queryClient,
  );
</script>

<Modal title="Debug trade`" bind:open outsideclose size="sm">
  {txHash}
  {rpcUrl}
  {#if $debugQuery.isLoading}
    Loading...
  {/if}
  {#if $debugQuery.isError}
    Error: {$debugQuery.error}
  {/if}
  {#if $debugQuery.data}
    {JSON.stringify($debugQuery.data)}
  {/if}
</Modal>
