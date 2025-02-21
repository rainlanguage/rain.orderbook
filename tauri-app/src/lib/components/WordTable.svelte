<script lang="ts">
  import type { AuthoringMetaV2 } from '@rainlanguage/orderbook/common';
  import {
    Input,
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
  } from 'flowbite-svelte';
  import Fuse from 'fuse.js';

  export let authoringMeta: AuthoringMetaV2;
  export let pragma: string;

  let search: string;

  const fuse = new Fuse(authoringMeta.words, {
    keys: ['word', 'description'],
    ignoreLocation: true,
    threshold: 0.0,
  });

  $: filteredWords = search ? fuse.search(search).map((res) => res.item) : authoringMeta.words;
</script>

<Table
  divClass="cursor-pointer rounded-lg dark:border-none border h-[500px] overflow-y-scroll relative w-[450px] bg-white dark:bg-gray-800"
  data-testid={`word-table-${pragma}`}
>
  <TableHead theadClass="sticky top-0">
    <TableHeadCell>
      <div class="flex flex-col text-xs font-normal">
        <div data-testid="pragma" class="mb-3 mt-1">
          From {pragma}
        </div>
        <Input data-testid="search-input" placeholder="Search words" bind:value={search} />
      </div>
    </TableHeadCell>
  </TableHead>
  <TableBody tableBodyClass="w-full">
    {#if filteredWords.length === 0}
      <TableBodyRow>
        <TableBodyCell>
          <div data-testid="no-results-msg" class="text-center text-gray-500">No words found</div>
        </TableBodyCell>
      </TableBodyRow>
    {:else}
      {#each filteredWords as word}
        <TableBodyRow>
          <TableBodyCell>
            <div class="flex flex-col gap-y-2">
              <div data-testid="word">{word.word}</div>
              <div data-testid="description" class="whitespace-normal text-gray-500">
                {word.description}
              </div>
            </div>
          </TableBodyCell>
        </TableBodyRow>
      {/each}
    {/if}
  </TableBody>
</Table>
