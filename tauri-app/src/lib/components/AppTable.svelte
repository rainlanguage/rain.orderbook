<script lang="ts" generics="T">
  import { Spinner, Table, TableBody, TableBodyRow, TableHead } from 'flowbite-svelte';
  import { FileCsvOutline } from 'flowbite-svelte-icons';
  import ButtonsPagination from '$lib/components/ButtonsPagination.svelte';
  import type { ListStore } from '$lib/storesGeneric/listStore';
  import { ButtonLoading } from '@rainlanguage/ui-components';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  // eslint-disable-next-line no-undef
  export let listStore: ListStore<T>;
  export let emptyMessage: string = 'None found';
  export let rowHoverable = true;
  export let enableCsvExport = true;

  listStore.fetchFirst();
</script>

{#if $listStore.isFetchingFirst}
  <div class="flex h-16 w-full items-center justify-center">
    <Spinner class="h-8 w-8" color="white" />
  </div>
{:else if $listStore.currentPage.length === 0}
  <div class="text-center text-gray-900 dark:text-white">{emptyMessage}</div>
{:else}
  <Table
    divClass="cursor-pointer rounded-lg overflow-hidden dark:border-none border"
    hoverable={rowHoverable}
  >
    <TableHead>
      <slot name="head" {listStore} />
    </TableHead>
    <TableBody>
      {#each $listStore.currentPage as item}
        <TableBodyRow
          on:click={() => {
            dispatch('clickRow', { item });
          }}
        >
          <slot name="bodyRow" {item} />
        </TableBodyRow>
      {/each}
    </TableBody>
  </Table>

  {#if enableCsvExport}
    <div class="mt-2 flex justify-between">
      <ButtonLoading
        size="xs"
        color="light"
        on:click={() => listStore.exportCsv()}
        loading={$listStore.isExporting}
      >
        <FileCsvOutline class="mr-2 h-4 w-4" />
        Export to CSV
      </ButtonLoading>
      <ButtonsPagination
        index={$listStore.index + 1}
        on:previous={listStore.fetchPrev}
        on:next={listStore.fetchNext}
        loading={$listStore.isFetching}
      />
    </div>
  {/if}
{/if}
