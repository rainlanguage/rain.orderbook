<script lang="ts" generics="T">
  import {
    Table,
    TableBody,
    TableBodyRow,
    TableHead,
  } from 'flowbite-svelte';
  import {  FileCsvOutline } from 'flowbite-svelte-icons';
  import ButtonsPagination from '$lib/components/ButtonsPagination.svelte';
  import type { PaginatedCachedStore } from '$lib/storesGeneric/listStore';
  import ButtonLoading from './ButtonLoading.svelte';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  export let listStore: PaginatedCachedStore<T>;
  export let emptyMessage: string = "None found"
  export let rowHoverable = true;
  export let enableCsvExport = true;
</script>

{#if $listStore.currentPage.length === 0}
  <div class="text-center text-gray-900 dark:text-white">{emptyMessage}</div>
{:else}
  <Table divClass="cursor-pointer" hoverable={rowHoverable}>
    <TableHead>
      <slot name="head" {listStore}></slot>
    </TableHead>
    <TableBody>
      {#each $listStore.currentPage as item}
        <TableBodyRow on:click={() => { dispatch('clickRow', {item}) }}>
          <slot name="bodyRow" {item}></slot>
        </TableBodyRow>
      {/each}
    </TableBody>
  </Table>

  {#if enableCsvExport}
    <div class="flex justify-between mt-2">
      <ButtonLoading size="xs" color="blue" on:click={() => listStore.exportCsv()} loading={$listStore.isExporting}>
        <FileCsvOutline class="w-4 h-4 mr-2"/>
        Export to CSV
      </ButtonLoading>
      <ButtonsPagination index={$listStore.index} on:previous={listStore.fetchPrev} on:next={listStore.fetchNext} loading={$listStore.isFetching} />
    </div>
  {/if}
{/if}