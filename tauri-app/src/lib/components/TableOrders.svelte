<script lang="ts">
  import { goto } from "$app/navigation";
  import type { Order } from "$lib/typeshare/orders";
  import { formatTimestampSecondsAsLocal } from "$lib/utils/time";
  import { Table, TableHead, TableHeadCell, TableBody, TableBodyRow, TableBodyCell, Badge, Button, Dropdown, DropdownItem,} from "flowbite-svelte";
  import { DotsVerticalOutline } from "flowbite-svelte-icons";
  import ModalOrderRemove from "$lib/components/ModalOrderRemove.svelte";
  import { walletAddressMatchesOrBlank } from "$lib/stores/settings";

  export let orders: Order[] = [];
  let removeModalOrderId: string;
  let showRemoveModal = false;

  function gotoOrder(id: string) {
    goto(`/orders/${id}`);
  }

  function confirmRemoveOrder(id: string) {
    removeModalOrderId = id;
    showRemoveModal = true;
  }
</script>

{#if orders.length === 0}
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
      {#each orders as order}
        <TableBodyRow on:click={() => gotoOrder(order.id)}>
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
              <DropdownItem on:click={(e) => {e.stopPropagation(); confirmRemoveOrder(order.id);}}>Remove</DropdownItem>
            </Dropdown>
          {/if}
        </TableBodyRow>
      {/each}
    </TableBody>
  </Table>

  <ModalOrderRemove bind:open={showRemoveModal} orderId={removeModalOrderId} />
{/if}