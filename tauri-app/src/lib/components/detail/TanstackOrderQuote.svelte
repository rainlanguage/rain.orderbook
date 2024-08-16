<script lang="ts" generics="T">
  import { QKEY_ORDER_QUOTE } from '$lib/queries/keys';
  import { batchOrderQuotes } from '$lib/services/order';
  import { formatUnits } from 'viem';
  import CardProperty from '../CardProperty.svelte';

  import { createQuery } from '@tanstack/svelte-query';
  import { Button, Spinner } from 'flowbite-svelte';

  export let orderHash: string;

  $: orderQuoteQuery = createQuery({
    queryKey: [QKEY_ORDER_QUOTE + orderHash],
    queryFn: () => batchOrderQuotes([orderHash]),
    enabled: !!orderHash,
    refetchInterval: 10000,
  });
</script>

{#if $orderQuoteQuery.isFetching || $orderQuoteQuery.isLoading}
  <div class="flex h-16 w-full items-center justify-center">
    <Spinner class="h-8 w-8" color="white" data-testid="loadingSpinner" />
  </div>
{:else if $orderQuoteQuery.data}
  <div class="grid grid-cols-2 gap-x-2">
    <CardProperty>
      <svelte:fragment slot="key">Maximum output</svelte:fragment>
      <svelte:fragment slot="value">
        {formatUnits(BigInt($orderQuoteQuery.data[0].maxOutput), 18)}
      </svelte:fragment>
    </CardProperty>
    <CardProperty>
      <svelte:fragment slot="key">Price</svelte:fragment>
      <svelte:fragment slot="value">
        {formatUnits(BigInt($orderQuoteQuery.data[0].ratio), 18)}
      </svelte:fragment>
    </CardProperty>
  </div>
  <div>
    <Button on:click={() => $orderQuoteQuery.refetch()}>Refresh Quote</Button>
  </div>
{:else}
  <div data-testid="emptyMessage" class="text-gray-900 dark:text-white">
    {'Max output and price not found'}
  </div>
{/if}
