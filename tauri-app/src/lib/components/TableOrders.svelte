<script lang="ts">
  import {
    Button,
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
    Badge,
    Dropdown,
    DropdownItem,
  } from 'flowbite-svelte';
  import { DotsVerticalOutline, FileCsvOutline } from 'flowbite-svelte-icons';
  import { goto } from '$app/navigation';
  import { orderRemove } from '$lib/utils/orderRemove';
  import { formatTimestampSecondsAsLocal } from '$lib/utils/time';
  import { walletAddressMatchesOrBlank } from '$lib/stores/settings';
  import ButtonsPagination from '$lib/components/ButtonsPagination.svelte';
  import type { PaginatedCachedStore } from '$lib/stores/paginatedStore';
  import type { Order } from '$lib/typeshare/ordersList';
  import ButtonLoading from './ButtonLoading.svelte';
  import Hash from './Hash.svelte';
  import { HashType } from '$lib/utils/hash';

  export let ordersList: PaginatedCachedStore<Order>;
</script>

{#if $ordersList.currentPage.length === 0}
  <div class="text-center text-gray-900 dark:text-white">No Orders found</div>
{:else}
  <Table divClass="cursor-pointer" hoverable={true}>
    <TableHead>
      <TableHeadCell>Active</TableHeadCell>
      <TableHeadCell>Order ID</TableHeadCell>
      <TableHeadCell>Owner</TableHeadCell>
      <TableHeadCell>Created At</TableHeadCell>
      <TableHeadCell>Input Token(s)</TableHeadCell>
      <TableHeadCell>Output Token(s)</TableHeadCell>
      <TableHeadCell padding="px-0"></TableHeadCell>
    </TableHead>
    <TableBody>
      {#each $ordersList.currentPage as order}
        <TableBodyRow on:click={() => goto(`/orders/${order.id}`)}>
          <TableBodyCell tdClass="px-4 py-2">
            {#if order.order_active}
              <Badge color="green">Active</Badge>
            {:else}
              <Badge color="yellow">Inactive</Badge>
            {/if}
          </TableBodyCell>
          <TableBodyCell tdClass="break-all px-4 py-2"><Hash type={HashType.Identifier} value={order.id} /></TableBodyCell>
          <TableBodyCell tdClass="break-all px-4 py-2"><Hash type={HashType.Wallet} value={order.owner.id} /></TableBodyCell>
          <TableBodyCell tdClass="break-word px-4 py-2">
            {formatTimestampSecondsAsLocal(BigInt(order.timestamp))}
          </TableBodyCell>
          <TableBodyCell tdClass="break-word p-2">
            {order.valid_inputs?.map((t) => t.token.symbol)}
          </TableBodyCell>
          <TableBodyCell tdClass="break-word p-2">
            {order.valid_outputs?.map((t) => t.token.symbol)}
          </TableBodyCell>
          <TableBodyCell tdClass="px-0">
            {#if $walletAddressMatchesOrBlank(order.owner.id) && order.order_active}
              <Button color="alternative" outline={false} id={`order-menu-${order.id}`} class="border-none px-2 mr-2" on:click={(e)=> {e.stopPropagation();}}>
                <DotsVerticalOutline class="dark:text-white"/>
              </Button>
            {/if}
          </TableBodyCell>
          {#if $walletAddressMatchesOrBlank(order.owner.id) && order.order_active}
            <Dropdown placement="bottom-end" triggeredBy={`#order-menu-${order.id}`}>
              <DropdownItem on:click={(e) => {e.stopPropagation(); orderRemove(order.id);}}>Remove</DropdownItem>
            </Dropdown>
          {/if}
        </TableBodyRow>
      {/each}
    </TableBody>
  </Table>

  <div class="flex justify-between mt-2">
    <ButtonLoading size="xs" color="blue" on:click={() => ordersList.exportCsv()} loading={$ordersList.isExporting}>
      <FileCsvOutline class="w-4 h-4 mr-2"/>
      Export to CSV
    </ButtonLoading>
    <ButtonsPagination index={$ordersList.index} on:previous={ordersList.fetchPrev} on:next={ordersList.fetchNext} loading={$ordersList.isFetching} />
  </div>
{/if}