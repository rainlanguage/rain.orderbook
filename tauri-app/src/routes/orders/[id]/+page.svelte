<script lang="ts">
  import CardProperty from './../../../lib/components/CardProperty.svelte';
  import { Button, TabItem, Tabs } from 'flowbite-svelte';
  import { orderDetail, useOrderTakesList } from '$lib/stores/order';
  import { walletAddressMatchesOrBlank } from '$lib/stores/wallets';
  import BadgeActive from '$lib/components/BadgeActive.svelte';
  import { formatTimestampSecondsAsLocal } from '$lib/utils/time';
  import ButtonVaultLink from '$lib/components/ButtonVaultLink.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import { page } from '$app/stores';
  import Hash from '$lib/components/Hash.svelte';
  import { HashType } from '$lib/types/hash';
  import { sortBy } from 'lodash';
  import LightweightChartLine from '$lib/components/LightweightChartLine.svelte';
  import PageContentDetail from '$lib/components/PageContentDetail.svelte';
  import CodeMirrorRainlang from '$lib/components/CodeMirrorRainlang.svelte';
  import { colorTheme } from '$lib/stores/darkMode';
  import ModalExecute from '$lib/components/ModalExecute.svelte';
  import { orderRemove, orderRemoveCalldata } from '$lib/services/order';
  import { ethersExecute } from '$lib/services/ethersTx';
  import { orderbookAddress } from '$lib/stores/settings';
  import { toasts } from '$lib/stores/toasts';
  import { reportErrorToSentry } from '$lib/services/sentry';
  import {
    prepareHistoricalOrderChartData,
    type HistoricalOrderChartData,
  } from '$lib/services/historicalOrderCharts';
  import { formatEthersTransactionError } from '$lib/utils/transaction';
  import TakeOrdersTable from '$lib/components/TakeOrdersTable.svelte';

  let openOrderRemoveModal = false;
  let isSubmitting = false;
  let orderTakesListChartData: HistoricalOrderChartData = [];

  const orderTakesList = useOrderTakesList($page.params.id);

  $: order = $orderDetail.data[$page.params.id]?.order;
  $: orderRainlang = $orderDetail.data[$page.params.id]?.rainlang;
  $: orderTakesListChartData = prepareHistoricalOrderChartData($orderTakesList.all, $colorTheme);

  $: orderTakesListChartDataSorted = sortBy(orderTakesListChartData, (d) => d.time);

  orderDetail.refetch($page.params.id);
  orderTakesList.fetchAll(0);

  async function executeLedger() {
    isSubmitting = true;
    try {
      await orderRemove(order.id);
    } catch (e) {
      reportErrorToSentry(e);
    }
    isSubmitting = false;
  }

  async function executeWalletconnect() {
    isSubmitting = true;
    try {
      const calldata = (await orderRemoveCalldata(order.id)) as Uint8Array;
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

<PageHeader title="Order" />

<PageContentDetail
  isFetching={$orderDetail.isFetching}
  isEmpty={order === undefined}
  emptyMessage="Order not found"
>
  <svelte:fragment slot="top">
    <div class="flex gap-x-4 text-3xl font-medium dark:text-white">
      <div class="flex gap-x-2">
        <span class="font-light">Order</span>
        <Hash shorten value={order.order_hash} />
      </div>
      <BadgeActive active={order.active} large />
    </div>
    {#if order && $walletAddressMatchesOrBlank(order.owner) && order.active}
      <Button color="dark" on:click={() => (openOrderRemoveModal = true)}>Remove</Button>
    {/if}
  </svelte:fragment>
  <svelte:fragment slot="card">
    <div class="flex flex-col gap-y-6">
      <CardProperty>
        <svelte:fragment slot="key">Owner</svelte:fragment>
        <svelte:fragment slot="value">
          <Hash type={HashType.Wallet} shorten={false} value={order.owner} />
        </svelte:fragment>
      </CardProperty>

      <CardProperty>
        <svelte:fragment slot="key">Created</svelte:fragment>
        <svelte:fragment slot="value">
          {formatTimestampSecondsAsLocal(BigInt(order.timestamp_added))}
        </svelte:fragment>
      </CardProperty>

      <CardProperty>
        <svelte:fragment slot="key">Input vaults</svelte:fragment>
        <svelte:fragment slot="value">
          {#each order.inputs || [] as t}
            <ButtonVaultLink tokenVault={t} />
          {/each}
        </svelte:fragment>
      </CardProperty>

      <CardProperty>
        <svelte:fragment slot="key">Output vaults</svelte:fragment>
        <svelte:fragment slot="value">
          {#each order.outputs || [] as t}
            <ButtonVaultLink tokenVault={t} />
          {/each}
        </svelte:fragment>
      </CardProperty>
    </div>
  </svelte:fragment>
  <svelte:fragment slot="chart">
    <LightweightChartLine
      title="Trades"
      data={orderTakesListChartDataSorted}
      loading={$orderTakesList.isFetchingAll}
      emptyMessage="No trades found"
    />
  </svelte:fragment>
  <svelte:fragment slot="below">
    <Tabs
      style="underline"
      contentClass="mt-4"
      defaultClass="flex flex-wrap space-x-2 rtl:space-x-reverse mt-4"
    >
      <TabItem open title="Rainlang source">
        {#if orderRainlang}
          <div class="mb-8 overflow-hidden rounded-lg border dark:border-none">
            <CodeMirrorRainlang disabled={true} value={orderRainlang} />
          </div>
        {:else}
          <div class="w-full tracking-tight text-gray-900 dark:text-white">
            Rain source not included in order meta
          </div>
        {/if}
      </TabItem>
      <TabItem title="Trades">
        <TakeOrdersTable {orderTakesList} />
      </TabItem>
    </Tabs>
  </svelte:fragment>
</PageContentDetail>

<ModalExecute
  bind:open={openOrderRemoveModal}
  title="Remove Order"
  execButtonLabel="Remove Order"
  {executeLedger}
  {executeWalletconnect}
  bind:isSubmitting
/>
