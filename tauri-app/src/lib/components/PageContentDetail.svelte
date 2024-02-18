<script lang="ts" generics="T">
  import { Card, Spinner } from "flowbite-svelte";

  // eslint-disable-next-line no-undef
  export let isEmpty = false;
  export let isFetching = false;
  export let emptyMessage = "Not found";
</script>

{#if !isEmpty}
  <div class="w-full flex justify-center items-stretch flex-wrap space-x-0 lg:flex-nowrap lg:space-x-4 mb-8 space-y-8 lg:space-y-0">
    <Card class="space-y-8 grow-0 w-full relative" size="md">
      <slot name="card"></slot>
    </Card>
    <slot name="chart"></slot>
  </div>
  <div class="w-full">
    <slot name="below"></slot>
  </div>
{:else if isFetching}
  <div class="w-full h-16 flex justify-center items-center">
    <Spinner class="h-8 w-8" color="white" />
  </div>
{:else}
  <div class="text-center text-gray-900 dark:text-white">{emptyMessage}</div>
{/if}