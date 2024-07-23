<script lang="ts" generics="T">
  import type { CreateQueryResult } from '@tanstack/svelte-query';
  import { Spinner } from 'flowbite-svelte';

  // eslint-disable-next-line no-undef
  export let query: CreateQueryResult<T>;
  export let emptyMessage = 'Not found';
</script>

{#if $query.data}
  <div class="mb-6 flex items-end justify-between">
    <slot name="top" data={$query.data} />
  </div>
  <div class="grid grid-cols-3 gap-4">
    <div class="col-span-1 flex flex-col gap-y-6">
      <slot name="card" data={$query.data} />
    </div>
    <div class="col-span-2 min-h-[500px]">
      <slot name="chart" data={$query.data} />
    </div>
  </div>
  <div class="w-full">
    <slot name="below" />
  </div>
{:else if $query.isFetching || $query.isLoading}
  <div class="flex h-16 w-full items-center justify-center">
    <Spinner class="h-8 w-8" color="white" />
  </div>
{:else}
  <div class="text-center text-gray-900 dark:text-white">{emptyMessage}</div>
{/if}
