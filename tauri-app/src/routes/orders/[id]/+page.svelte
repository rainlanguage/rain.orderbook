<script lang="ts">
  import CardProperty from './../../../lib/components/CardProperty.svelte';
  import { TabItem, TableBodyCell, TableHeadCell, Tabs } from 'flowbite-svelte';
  import { orderDetail, useOrderTakesList } from '$lib/stores/order';
  import { walletAddressMatchesOrBlank } from '$lib/stores/wallets';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import BadgeActive from '$lib/components/BadgeActive.svelte';
  import { formatTimestampSecondsAsLocal, timestampSecondsToUTCTimestamp } from '$lib/utils/time';
  import ButtonVaultLink from '$lib/components/ButtonVaultLink.svelte';
  import { orderRemove } from '$lib/services/order';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import { page } from '$app/stores';
  import Hash from '$lib/components/Hash.svelte';
  import { HashType } from '$lib/types/hash';
  import AppTable from '$lib/components/AppTable.svelte';
  import { sortBy } from 'lodash';
  import LightweightChartLine from '$lib/components/LightweightChartLine.svelte';
  import PageContentDetail from '$lib/components/PageContentDetail.svelte';
  import CodeMirrorRainlang from '$lib/components/CodeMirrorRainlang.svelte';
  import type { UTCTimestamp } from 'lightweight-charts';
  import { colorTheme } from '$lib/stores/darkMode';

  let isSubmitting = false;
  let orderTakesListChartData:  { value: number; time: UTCTimestamp; color?: string }[] = [];

  const orderTakesList = useOrderTakesList($page.params.id);

  $: order = $orderDetail.data[$page.params.id]?.order;
  $: orderRainlang = $orderDetail.data[$page.params.id]?.rainlang;
  $: $orderTakesList.all, orderTakesListChartData = prepareChartData();

  async function remove() {
    isSubmitting = true;
    try {
      await orderRemove(order.id);
      // eslint-disable-next-line no-empty
    } catch (e) {}
    isSubmitting = false;
  }

  function prepareChartData() {
    const transformedData = $orderTakesList.all.map((d) => ({
      value: parseFloat(d.ioratio),
      time: timestampSecondsToUTCTimestamp(BigInt(d.timestamp)),
      color: '#4E4AF6',
    }));

    return sortBy(
      transformedData,
      (d) => d.time
    );
  }

  $: orderTakesListChartData = $orderTakesList.all.map((d) => ({
    value: parseFloat(d.ioratio),
    time: timestampSecondsToUTCTimestamp(BigInt(d.timestamp)),
    color: $colorTheme == 'dark' ? '#5178FF' : '#4E4AF6',
  }));
  $: orderTakesListChartDataSorted = sortBy(orderTakesListChartData, (d) => d.time);

  orderDetail.refetch($page.params.id);
  orderTakesList.fetchAll(0);
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
        <Hash shorten value={order.id} />
      </div>
      <BadgeActive active={order.order_active} large />
    </div>
    {#if order && $walletAddressMatchesOrBlank(order.owner.id) && order.order_active}
      <ButtonLoading color="dark" on:click={remove} loading={isSubmitting}>Remove</ButtonLoading>
    {/if}
  </svelte:fragment>
  <svelte:fragment slot="card">
    <div class="flex flex-col gap-y-6">
      <CardProperty>
        <svelte:fragment slot="key">Owner</svelte:fragment>
        <svelte:fragment slot="value">
          <Hash type={HashType.Wallet} shorten={false} value={order.owner.id} />
        </svelte:fragment>
      </CardProperty>

      <CardProperty>
        <svelte:fragment slot="key">Created</svelte:fragment>
        <svelte:fragment slot="value">
          {formatTimestampSecondsAsLocal(BigInt(order.timestamp))}
        </svelte:fragment>
      </CardProperty>

      <CardProperty>
        <svelte:fragment slot="key">Input vaults</svelte:fragment>
        <svelte:fragment slot="value">
          {#each order.valid_inputs || [] as t}
            <ButtonVaultLink tokenVault={t.token_vault} />
          {/each}
        </svelte:fragment>
      </CardProperty>

      <CardProperty>
        <svelte:fragment slot="key">Output vaults</svelte:fragment>
        <svelte:fragment slot="value">
          {#each order.valid_outputs || [] as t}
            <ButtonVaultLink tokenVault={t.token_vault} />
          {/each}
        </svelte:fragment>
      </CardProperty>
    </div>
  </svelte:fragment>
  <svelte:fragment slot="chart">
    <LightweightChartLine
      title="Trades"
      data={orderTakesListChartData}
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
        <AppTable listStore={orderTakesList} emptyMessage="No trades found" rowHoverable={false}>
          <svelte:fragment slot="head">
            <TableHeadCell padding="p-4">Date</TableHeadCell>
            <TableHeadCell padding="p-0">Sender</TableHeadCell>
            <TableHeadCell padding="p-0">Transaction Hash</TableHeadCell>
            <TableHeadCell padding="p-0">Input</TableHeadCell>
            <TableHeadCell padding="p-0">Output</TableHeadCell>
            <TableHeadCell padding="p-0">IO Ratio</TableHeadCell>
          </svelte:fragment>

          <svelte:fragment slot="bodyRow" let:item>
            <TableBodyCell tdClass="px-4 py-2">
              {formatTimestampSecondsAsLocal(BigInt(item.timestamp))}
            </TableBodyCell>
            <TableBodyCell tdClass="break-all py-2 min-w-32">
              <Hash type={HashType.Wallet} value={item.sender.id} />
            </TableBodyCell>
            <TableBodyCell tdClass="break-all py-2 min-w-32">
              <Hash type={HashType.Transaction} value={item.transaction.id} />
            </TableBodyCell>
            <TableBodyCell tdClass="break-all py-2">
              {item.input_display}
              {item.input_token.symbol}
            </TableBodyCell>
            <TableBodyCell tdClass="break-all py-2">
              {item.output_display}
              {item.output_token.symbol}
            </TableBodyCell>
            <TableBodyCell tdClass="break-all py-2">
              {item.ioratio}
              {item.input_token.symbol}/{item.output_token.symbol}
            </TableBodyCell>
          </svelte:fragment>
        </AppTable>
      </TabItem>
    </Tabs>
  </svelte:fragment>
</PageContentDetail>
