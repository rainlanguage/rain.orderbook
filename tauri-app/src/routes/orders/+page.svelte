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
  } from 'flowbite-svelte';
  import { goto } from '$app/navigation';
  import { ordersList } from '$lib/stores/ordersList';
  import dayjs from 'dayjs';
  import utc from 'dayjs/plugin/utc';
  import bigIntSupport from 'dayjs/plugin/bigIntSupport';
  dayjs.extend(utc);
  dayjs.extend(bigIntSupport);

  let showDepositModal = false;
  let showWithdrawModal = false;

  function gotoOrder(id: string) {
    goto(`/orders/${id}`);
  }

  function toggleDepositModal() {
    showDepositModal = !showDepositModal;
  }
  function toggleWithdrawModal() {
    showWithdrawModal = !showWithdrawModal;
  }

  redirectIfSettingsNotDefined();
  ordersList.refetch();
</script>

<div class="flex w-full">
  <div class="flex-1"></div>
  <h1 class="flex-0 mb-8 text-4xl font-bold text-gray-900 dark:text-white">Orders</h1>
  <div class="flex-1">
    <div class="flex justify-end space-x-2">
      <Button color="green" size="xs" on:click={toggleDepositModal}>Add</Button>
      <Button color="blue" size="xs" on:click={toggleWithdrawModal}>Remove</Button>
    </div>
  </div>
</div>

{#if $ordersList.length === 0}
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
    </TableHead>
    <TableBody>
      {#each $ordersList as order}
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
            {dayjs(BigInt(order.timestamp) * BigInt('1000'))
              .utc(true)
              .local()
              .format('DD/MM/YYYY h:mm A')}
          </TableBodyCell>
          <TableBodyCell tdClass="break-word p-2">
            {order.valid_inputs?.map((t) => t.token.symbol)}
          </TableBodyCell>
          <TableBodyCell tdClass="break-word p-2">
            {order.valid_outputs?.map((t) => t.token.symbol)}
          </TableBodyCell>
        </TableBodyRow>
      {/each}
    </TableBody>
  </Table>
{/if}
