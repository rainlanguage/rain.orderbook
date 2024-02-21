<script lang="ts" generics="T">
  import { Spinner } from 'flowbite-svelte';

  // eslint-disable-next-line no-undef
  export let isEmpty = false;
  export let isFetching = false;
  export let emptyMessage = 'Not found';
</script>

{#if !isEmpty}
  <div class="mb-6 flex items-end justify-between">
    <slot name="top" />
  </div>
  <div class="grid grid-cols-3 gap-4">
    <div class="col-span-1">
      <slot name="card" />
    </div>
    <div class="col-span-2 min-h-[500px]">
      <slot name="chart" />
    </div>
  </div>
  <div class="w-full">
    <slot name="below" />
  </div>
{:else if isFetching}
  <div class="flex h-16 w-full items-center justify-center">
    <Spinner class="h-8 w-8" color="white" />
  </div>
{:else}
  <div class="text-center text-gray-900 dark:text-white">{emptyMessage}</div>
{/if}
