<script lang="ts">
  import { ordersList } from '$lib/stores/order';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import { DotsVerticalOutline } from 'flowbite-svelte-icons';
  import { goto } from '$app/navigation';
  import { formatTimestampSecondsAsLocal } from '$lib/utils/time';
  import { walletAddressMatchesOrBlank } from '$lib/stores/wallets';
  import Hash from '$lib/components/Hash.svelte';
  import { HashType } from '$lib/types/hash';
  import AppTable from '$lib/components/AppTable.svelte';
  import {
    orderbookAddress,
    resetActiveNetworkRef,
    resetActiveOrderbookRef,
    subgraphUrl,
    activeOrderbook,
  } from '$lib/stores/settings';
  import ModalExecute from '$lib/components/ModalExecute.svelte';
  import { orderRemove, orderRemoveCalldata } from '$lib/services/order';
  import { ethersExecute } from '$lib/services/ethersTx';
  import { toasts } from '$lib/stores/toasts';
  import {
    Button,
    TableBodyCell,
    TableHeadCell,
    Badge,
    Dropdown,
    DropdownItem,
    Spinner,
  } from 'flowbite-svelte';
  import { reportErrorToSentry } from '$lib/services/sentry';
  import ListViewOrderbookSelector from '$lib/components/ListViewOrderbookSelector.svelte';
  import { onMount } from 'svelte';
  import { formatEthersTransactionError } from '$lib/utils/transaction';

  onMount(async () => {
    if (!$activeOrderbook) {
      await resetActiveNetworkRef();
      resetActiveOrderbookRef();
    }
  });

  $: $subgraphUrl, $ordersList?.fetchFirst();
  let openOrderRemoveModal = false;
  let id: string;
  let isSubmitting = false;

  async function executeLedger() {
    isSubmitting = true;
    try {
      await orderRemove(id);
    } catch (e) {
      reportErrorToSentry(e);
    }
    isSubmitting = false;
  }
  async function executeWalletconnect() {
    isSubmitting = true;
    try {
      const calldata = (await orderRemoveCalldata(id)) as Uint8Array;
      const tx = await ethersExecute(calldata, $orderbookAddress!);
      toasts.success('Transaction sent successfully!');
      await tx.wait(1);
    } catch (e) {
      reportErrorToSentry(e);
      toasts.error(formatEthersTransactionError(e));
    }
    isSubmitting = false;
  }
</script>

<PageHeader title="Orders" />

{#if $ordersList === undefined}
  <div class="flex h-16 w-full items-center justify-center">
    <Spinner class="h-8 w-8" color="white" />
  </div>
{:else}
  <div class="flex w-full justify-between py-4">
    <div class="text-3xl font-medium dark:text-white">Orders</div>
    <ListViewOrderbookSelector />
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
            }}>Remove</DropdownItem
          >
        </Dropdown>
      {/if}
    </svelte:fragment>
  </AppTable>
{/if}

<ModalExecute
  bind:open={openOrderRemoveModal}
  title="Remove Order"
  execButtonLabel="Remove Order"
  {executeLedger}
  {executeWalletconnect}
  bind:isSubmitting
/>
