<script lang="ts">
  import { ordersList } from '$lib/stores/order';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import {
    Button,
    TableBodyCell,
    TableHeadCell,
    Badge,
    Dropdown,
    DropdownItem,
    Spinner,
  } from 'flowbite-svelte';
  import { DotsVerticalOutline } from 'flowbite-svelte-icons';
  import { goto } from '$app/navigation';
  // import { orderRemove } from '$lib/services/order';
  import { formatTimestampSecondsAsLocal } from '$lib/utils/time';
  import { walletAddressMatchesOrBlank } from '$lib/stores/wallets';
  import Hash from '$lib/components/Hash.svelte';
  import { HashType } from '$lib/types/hash';
  import AppTable from '$lib/components/AppTable.svelte';
  import { subgraphUrl } from '$lib/stores/settings';
  import ModalOrderRemove from '$lib/components/ModalOrderRemove.svelte';

  $: $subgraphUrl, $ordersList?.fetchFirst();
  let openOrderRemoveModal = false;
  let id: string;
</script>

<PageHeader title="Orders" />

{#if $ordersList === undefined}
  <div class="flex h-16 w-full items-center justify-center">
    <Spinner class="h-8 w-8" color="white" />
  </div>
{:else}
  <div class="flex w-full justify-between py-4">
    <div class="text-3xl font-medium dark:text-white">Orders</div>
    <Button color="green" href="/orders/add">Add Order</Button>
  </div>

  <AppTable
    listStore={$ordersList}
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
              id = item.id;
              openOrderRemoveModal = true;
              // orderRemove(item.id);
            }}>Remove</DropdownItem
          >
        </Dropdown>
      {/if}
    </svelte:fragment>
  </AppTable>
{/if}

<ModalOrderRemove bind:open={openOrderRemoveModal} id={id}/>