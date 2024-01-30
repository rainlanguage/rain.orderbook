<script lang="ts">
  import { redirectIfSettingsNotDefined } from '$lib/utils/redirect';
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
  import { DotsVerticalOutline } from 'flowbite-svelte-icons';
  import { goto } from '$app/navigation';
  import { ordersPage, refetchOrdersPage } from '$lib/stores/ordersList';
  import { orderRemove } from '$lib/utils/orderRemove';
  import { formatTimestampSecondsAsLocal } from '$lib/utils/time';
  import { walletAddressMatchesOrBlank } from '$lib/stores/settings';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import ButtonsPagination from '$lib/components/ButtonsPagination.svelte';
  import { toasts } from '$lib/stores/toasts';
  import { ToastMessageType } from '$lib/typeshare/toast';

  let page = 1;
  let isFetchingNext = false;
  let isFetchingPrev = false;

  async function prevPage() {
    if(page <= 1) return;

    isFetchingPrev = true;
    try {
      const req = refetchOrdersPage(page-1);
      if($ordersPage(page-1).length === 0) {
        await req;
      }
      page -=1;
    // eslint-disable-next-line no-empty
    } catch(e) {}
    isFetchingPrev = false;
  }
  async function nextPage() {
    isFetchingNext = true;
    try {
      const req = refetchOrdersPage(page+1);
      if($ordersPage(page+1).length === 0) {
        await req;
      }
      page +=1;
    } catch(e) {
      toasts.add({
        message_type: ToastMessageType.Error,
        text: "No more pages"
      });
    }
    isFetchingNext = false;
  }

  redirectIfSettingsNotDefined();
  refetchOrdersPage(page);
</script>

<PageHeader title="Orders">
  <svelte:fragment slot="actions">
    <Button color="green" size="xs">Add</Button>
  </svelte:fragment>
</PageHeader>

{#if $ordersPage(page).length === 0}
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
      {#each $ordersPage(page) as order}
        <TableBodyRow on:click={() => goto(`/orders/${order.id}`)}>
          <TableBodyCell tdClass="px-4 py-2">
            {#if order.order_active}
              <Badge color="green">Active</Badge>
            {:else}
              <Badge color="yellow">Inactive</Badge>
            {/if}
          </TableBodyCell>
          <TableBodyCell tdClass="break-all px-4 py-2">{order.id}</TableBodyCell>
          <TableBodyCell tdClass="break-all px-4 py-2">{order.owner.id}</TableBodyCell>
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

  <div class="flex justify-end mt-2">
    <ButtonsPagination page={page} on:previous={prevPage} on:next={nextPage} nextLoading={isFetchingNext} prevLoading={isFetchingPrev} />
  </div>
{/if}