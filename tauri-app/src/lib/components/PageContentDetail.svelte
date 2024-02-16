<script lang="ts" generics="T">
  import { Card } from "flowbite-svelte";

  // eslint-disable-next-line no-undef
  export let item: T;
  export let emptyMessage = "Not found";
</script>


{#if item === undefined}
  <div class="text-center text-gray-900 dark:text-white">{emptyMessage}</div>
{:else}
  <div class="w-full flex justify-center items-stretch flex-wrap space-x-0 lg:flex-nowrap lg:space-x-4 mb-8 space-y-8 lg:space-y-0">
    <Card class="space-y-8 grow-0 w-full relative" size="md">
      <slot name="card" {item}></slot>
    </Card>

    <slot name="chart" {item}></slot>
  </div>


  <div class="w-full">
    <slot name="below" {item}></slot>
  </div>
{/if}