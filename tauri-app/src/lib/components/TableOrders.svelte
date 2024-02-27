<script lang="ts">
  import {
    Button,
    TableBodyCell,
    TableHeadCell,
    Badge,
    Dropdown,
    DropdownItem,
  } from 'flowbite-svelte';
  import { DotsVerticalOutline } from 'flowbite-svelte-icons';
  import { goto } from '$app/navigation';
  import { orderRemove } from '$lib/services/order';
  import { formatTimestampSecondsAsLocal } from '$lib/utils/time';
  import { walletAddressMatchesOrBlank } from '$lib/stores/settings';
  import type { ListStore } from '$lib/storesGeneric/listStore';
  import type { Order } from '$lib/typeshare/ordersList';
  import Hash from './Hash.svelte';
  import { HashType } from '$lib/types/hash';
  import AppTable from '$lib/components/AppTable.svelte';

  export let ordersList: ListStore<Order>;
</script>

<AppTable
  listStore={ordersList}
  emptyMessage="No Orders Found"
  on:clickRow={(e) => {
    goto(`/orders/${e.detail.item.id}`);
  }}
>
  <svelte:fragment slot="head">
    <TableHeadCell padding="p-4">Active</TableHeadCell>
    <TableHeadCell padding="p-4">Order ID</TableHeadCell>
    <TableHeadCell padding="p-4">Owner</TableHeadCell>
    <TableHeadCell padding="p-4">Created At</TableHeadCell>
    <TableHeadCell padding="px-2 py-4">Input Token(s)</TableHeadCell>
    <TableHeadCell padding="px-2 py-4">Output Token(s)</TableHeadCell>
    <TableHeadCell padding="px-4 py-4"></TableHeadCell>
  </svelte:fragment>

  <svelte:fragment slot="bodyRow" let:item>
    <TableBodyCell tdClass="px-4 py-2">
      {#if item.order_active}
        <Badge color="green">Active</Badge>
      {:else}
        <Badge color="yellow">Inactive</Badge>
      {/if}
    </TableBodyCell>
    <TableBodyCell tdClass="break-all px-4 py-4"
      ><Hash type={HashType.Identifier} value={item.id} /></TableBodyCell
    >
    <TableBodyCell tdClass="break-all px-4 py-2"
      ><Hash type={HashType.Wallet} value={item.owner.id} /></TableBodyCell
    >
    <TableBodyCell tdClass="break-word px-4 py-2">
      {formatTimestampSecondsAsLocal(BigInt(item.timestamp))}
    </TableBodyCell>
    <TableBodyCell tdClass="break-word p-2">
      {item.valid_inputs?.map((t) => t.token.symbol)}
    </TableBodyCell>
    <TableBodyCell tdClass="break-word p-2">
      {item.valid_outputs?.map((t) => t.token.symbol)}
    </TableBodyCell>
    <TableBodyCell tdClass="px-0 text-right">
      {#if $walletAddressMatchesOrBlank(item.owner.id) && item.order_active}
        <Button
          color="alternative"
          outline={false}
          id={`order-menu-${item.id}`}
          class="mr-2 border-none px-2"
          on:click={(e) => {
            e.stopPropagation();
          }}
        >
          <DotsVerticalOutline class="dark:text-white" />
        </Button>
      {/if}
    </TableBodyCell>
    {#if $walletAddressMatchesOrBlank(item.owner.id) && item.order_active}
      <Dropdown placement="bottom-end" triggeredBy={`#order-menu-${item.id}`}>
        <DropdownItem
          on:click={(e) => {
            e.stopPropagation();
            orderRemove(item.id);
          }}>Remove</DropdownItem
        >
      </Dropdown>
    {/if}
  </svelte:fragment>
</AppTable>
