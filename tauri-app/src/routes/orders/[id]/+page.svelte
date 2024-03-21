<script lang="ts">
  import CardProperty from './../../../lib/components/CardProperty.svelte';
  import { Button, TabItem, TableBodyCell, TableHeadCell, Tabs } from 'flowbite-svelte';
  import { orderDetail, useOrderTakesList } from '$lib/stores/order';
  import { walletAddressMatchesOrBlank } from '$lib/stores/wallets';
  import BadgeActive from '$lib/components/BadgeActive.svelte';
  import { formatTimestampSecondsAsLocal, timestampSecondsToUTCTimestamp } from '$lib/utils/time';
  import ButtonVaultLink from '$lib/components/ButtonVaultLink.svelte';
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
  import ModalExecute from '$lib/components/ModalExecute.svelte';
  import { orderRemove, orderRemoveCalldata } from '$lib/services/order';
  import { ethersExecute } from '$lib/services/ethersTx';
  import { orderbookAddress } from '$lib/stores/settings';
  import { toasts } from '$lib/stores/toasts';

  let openOrderRemoveModal = false;
  let isSubmitting = false;
  type ChartData = { value: number; time: UTCTimestamp; color?: string}[];
  let orderTakesListChartData: ChartData = [];

  const orderTakesList = useOrderTakesList($page.params.id);

  $: order = $orderDetail.data[$page.params.id]?.order;
  $: orderRainlang = $orderDetail.data[$page.params.id]?.rainlang;
  $: $orderTakesList.all, orderTakesListChartData = prepareChartData();

  function prepareChartData() {
    const transformedData = $orderTakesList.all.map((d) => ({
      value: parseFloat(d.ioratio),
      time: timestampSecondsToUTCTimestamp(BigInt(d.timestamp)),
      color: $colorTheme == 'dark' ? '#5178FF' : '#4E4AF6',
      outputAmount: +d.output_display
    }));

    // if we have multiple object in the array with the same timestamp, we need to merge them
    // we do this by taking the weighted average of the ioratio values for objects that share the same timestamp.
    const uniqueTimestamps = Array.from(new Set(transformedData.map((d) => d.time)));
    let finalData: ChartData = [];
    uniqueTimestamps.forEach((timestamp) => {
      const objectsWithSameTimestamp = transformedData.filter((d) => d.time === timestamp);
      if (objectsWithSameTimestamp.length > 1) {
        // calculate a weighted average of the ioratio values using the amount of the output token as the weight
        const ioratioSum = objectsWithSameTimestamp.reduce((acc, d) => acc + d.value * d.outputAmount, 0);
        const outputAmountSum = objectsWithSameTimestamp.reduce((acc, d) => acc + d.outputAmount, 0);
        const ioratioAverage = ioratioSum / outputAmountSum;
        finalData.push({
          value: ioratioAverage,
          time: timestamp,
          color: objectsWithSameTimestamp[0].color,
        });
      }
    });

    return sortBy(
      finalData,
      (d) => d.time
    );
  }
  $: orderTakesListChartDataSorted = sortBy(orderTakesListChartData, (d) => d.time);

  orderDetail.refetch($page.params.id);
  orderTakesList.fetchAll(0);

  async function executeLedger() {
    isSubmitting = true;
    try {
      await orderRemove(order.id);
      // eslint-disable-next-line no-empty
    } catch (e) {}
    isSubmitting = false;
  }

  async function executeWalletconnect() {
    isSubmitting = true;
    try {
      const calldata = await orderRemoveCalldata(order.id) as Uint8Array;
      const tx = await ethersExecute(calldata, $orderbookAddress!);
      toasts.success("Transaction sent successfully!");
      await tx.wait(1);
    } catch (e) {
      // eslint-disable-next-line no-console
      console.log(e);
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      if (typeof e === "object" && (e as any)?.reason) {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        toasts.error(`Transaction failed, reason: ${(e as any).reason}`);
      }
      else toasts.error("Transaction failed!");
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
        <Hash shorten value={order.id} />
      </div>
      <BadgeActive active={order.order_active} large />
    </div>
    {#if order && $walletAddressMatchesOrBlank(order.owner.id) && order.order_active}
      <Button color="dark" on:click={() => openOrderRemoveModal = true}>Remove</Button>
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

<ModalExecute
  bind:open={openOrderRemoveModal}
  title="Remove Order"
  execButtonLabel="Remove Order"
  {executeLedger}
  {executeWalletconnect}
  bind:isSubmitting={isSubmitting}
/>