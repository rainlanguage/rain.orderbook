<script lang="ts">
  import { redirectIfSettingsNotDefined } from '$lib/utils/redirect';
  import {
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
  } from 'flowbite-svelte';
  import { goto } from '$app/navigation';
  import { orderClearsList } from '$lib/stores/orderClearsList';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import ButtonsPagination from '$lib/components/ButtonsPagination.svelte';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import Hash from '$lib/components/Hash.svelte';
  import { HashType } from '$lib/utils/hash';
  import { formatTimestampSecondsAsLocal } from '$lib/utils/time';
  import { FileCsvOutline } from 'flowbite-svelte-icons';

  redirectIfSettingsNotDefined();
  orderClearsList.fetchFirst();
</script>

<PageHeader title="Order Clears" />


{#if $orderClearsList.currentPage.length === 0}
  <div class="text-center text-gray-900 dark:text-white">No Order Clears found</div>
{:else}
  <Table divClass="cursor-pointer" hoverable={true}>
    <TableHead>
      <TableHeadCell>Order Clear ID</TableHeadCell>
      <TableHeadCell>Cleared At</TableHeadCell>
      <TableHeadCell>Sender</TableHeadCell>
      <TableHeadCell>Clearer</TableHeadCell>
      <TableHeadCell>Bounty A</TableHeadCell>
      <TableHeadCell>Bounty B</TableHeadCell>
    </TableHead>
    <TableBody>
      {#each $orderClearsList.currentPage as orderClear}
        <TableBodyRow on:click={() => {goto(`/order-clears/${orderClear.id}`)}}>
          <TableBodyCell tdClass="break-all px-4 py-2"><Hash type={HashType.Identifier} value={orderClear.id} /></TableBodyCell>
          <TableBodyCell tdClass="break-word px-4 py-2">
            {formatTimestampSecondsAsLocal(BigInt(orderClear.timestamp))}
          </TableBodyCell>
          <TableBodyCell tdClass="break-all px-4 py-2 min-w-48"><Hash type={HashType.Wallet} value={orderClear.sender.id} /></TableBodyCell>
          <TableBodyCell tdClass="break-all px-4 py-2 min-w-48"><Hash type={HashType.Wallet} value={orderClear.clearer.id} /></TableBodyCell>
          <TableBodyCell tdClass="break-all px-4 py-2 min-w-48">{orderClear.bounty.bounty_amount_adisplay} {orderClear.bounty.bounty_token_a.symbol}</TableBodyCell>
          <TableBodyCell tdClass="break-all px-4 py-2 min-w-48">{orderClear.bounty.bounty_amount_bdisplay} {orderClear.bounty.bounty_token_b.symbol}</TableBodyCell>
        </TableBodyRow>
      {/each}
    </TableBody>
  </Table>

  <div class="flex justify-between mt-2">
    <ButtonLoading size="xs" color="blue" on:click={() => orderClearsList.exportCsv()} loading={$orderClearsList.isExporting}>
      <FileCsvOutline class="w-4 h-4 mr-2"/>
      Export to CSV
    </ButtonLoading>
    <ButtonsPagination index={$orderClearsList.index} on:previous={orderClearsList.fetchPrev} on:next={orderClearsList.fetchNext} loading={$orderClearsList.isFetching} />
  </div>
{/if}