<script lang="ts">
  import { QKEY_ORDERS } from '$lib/queries/keys';
  import { ordersList } from '$lib/queries/ordersList';
  import { createInfiniteQuery } from '@tanstack/svelte-query';
  import { DEFAULT_PAGE_SIZE, DEFAULT_REFRESH_INTERVAL } from '$lib/queries/constants';
  import TanstackAppTable from './TanstackAppTable.svelte';
  import { goto } from '$app/navigation';
  import ListViewOrderbookSelector from '../ListViewOrderbookSelector.svelte';
  import {
    Badge,
    Button,
    Dropdown,
    DropdownItem,
    TableBodyCell,
    TableHeadCell,
  } from 'flowbite-svelte';
  import { DotsVerticalOutline } from 'flowbite-svelte-icons';
  import { walletAddressMatchesOrBlank } from '$lib/stores/wallets';
  import Hash from '$lib/components/Hash.svelte';
  import { HashType } from '$lib/types/hash';
  import { subgraphUrl } from '$lib/stores/settings';
  import { formatTimestampSecondsAsLocal } from '$lib/utils/time';
  import { handleOrderRemoveModal } from '$lib/services/modal';
  import { activeWatchlist } from '$lib/stores/settings';

  $: query = createInfiniteQuery({
    queryKey: [QKEY_ORDERS],
    queryFn: ({ pageParam }) => {
      return ordersList($subgraphUrl, Object.values($activeWatchlist), pageParam);
    },
    initialPageParam: 0,
    getNextPageParam(lastPage, _allPages, lastPageParam) {
      return lastPage.length === DEFAULT_PAGE_SIZE ? lastPageParam + 1 : undefined;
    },
    refetchInterval: DEFAULT_REFRESH_INTERVAL,
    enabled: !!$subgraphUrl,
  });
</script>

{#if $query}
  <TanstackAppTable
    {query}
    emptyMessage="No Orders Found"
    on:clickRow={(e) => {
      goto(`/orders/${e.detail.item.id}`);
    }}
  >
    <svelte:fragment slot="title">
      <div class="flex w-full justify-between py-4">
        <div class="text-3xl font-medium dark:text-white">Orders</div>
        <ListViewOrderbookSelector />
      </div>
    </svelte:fragment>

    <svelte:fragment slot="head">
      <TableHeadCell data-testid="orderListHeadingActive" padding="p-4">Active</TableHeadCell>
      <TableHeadCell data-testid="orderListHeadingID" padding="p-4">Order</TableHeadCell>
      <TableHeadCell data-testid="orderListHeadingOwner" padding="p-4">Owner</TableHeadCell>
      <TableHeadCell data-testid="orderListHeadingOrderbook" padding="p-4">Orderbook</TableHeadCell>
      <TableHeadCell data-testid="orderListHeadingLastAdded" padding="p-4">Last Added</TableHeadCell
      >
      <TableHeadCell data-testid="orderListHeadingInputs" padding="px-2 py-4"
        >Input Token(s)</TableHeadCell
      >
      <TableHeadCell data-testid="orderListHeadingOutputs" padding="px-2 py-4"
        >Output Token(s)</TableHeadCell
      >
      <TableHeadCell padding="px-4 py-4"></TableHeadCell>
    </svelte:fragment>

    <svelte:fragment slot="bodyRow" let:item>
      <TableBodyCell data-testid="orderListRowActive" tdClass="px-4 py-2">
        {#if item.active}
          <Badge color="green">Active</Badge>
        {:else}
          <Badge color="yellow">Inactive</Badge>
        {/if}
      </TableBodyCell>
      <TableBodyCell data-testid="orderListRowID" tdClass="break-all px-4 py-4"
        ><Hash type={HashType.Identifier} value={item.order_hash} /></TableBodyCell
      >
      <TableBodyCell data-testid="orderListRowOwner" tdClass="break-all px-4 py-2"
        ><Hash type={HashType.Wallet} value={item.owner} /></TableBodyCell
      >
      <TableBodyCell data-testid="orderListRowOrderbook" tdClass="break-all px-4 py-2"
        ><Hash type={HashType.Identifier} value={item.orderbook.id} /></TableBodyCell
      >
      <TableBodyCell data-testid="orderListRowLastAdded" tdClass="break-word px-4 py-2">
        {formatTimestampSecondsAsLocal(BigInt(item.timestamp_added))}
      </TableBodyCell>
      <TableBodyCell data-testid="orderListRowInputs" tdClass="break-word p-2">
        {item.inputs?.map((t) => t.token.symbol)}
      </TableBodyCell>
      <TableBodyCell data-testid="orderListRowOutputs" tdClass="break-word p-2">
        {item.outputs?.map((t) => t.token.symbol)}
      </TableBodyCell>
      <TableBodyCell tdClass="px-0 text-right">
        {#if $walletAddressMatchesOrBlank(item.owner) && item.active}
          <Button
            color="alternative"
            outline={false}
            data-testid={`order-menu-${item.id}`}
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
      {#if $walletAddressMatchesOrBlank(item.owner) && item.active}
        <Dropdown placement="bottom-end" triggeredBy={`#order-menu-${item.id}`}>
          <DropdownItem
            on:click={(e) => {
              e.stopPropagation();
              handleOrderRemoveModal(item);
            }}>Remove</DropdownItem
          >
        </Dropdown>
      {/if}
    </svelte:fragment>
  </TanstackAppTable>
{/if}
