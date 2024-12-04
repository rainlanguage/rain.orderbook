<script lang="ts">
  import CardProperty from './../../../lib/components/CardProperty.svelte';
  import { Button, TabItem, Tabs } from 'flowbite-svelte';
  import { walletAddressMatchesOrBlank } from '$lib/stores/wallets';
  import { BadgeActive } from '@rainlanguage/ui-components';
  import { formatTimestampSecondsAsLocal } from '@rainlanguage/ui-components';
  import { ButtonVaultLink } from '@rainlanguage/ui-components';
  import { Hash, HashType } from '@rainlanguage/ui-components';

  import { CodeMirrorRainlang } from '@rainlanguage/ui-components';
  import { settings, orderbookAddress } from '$lib/stores/settings';
  import { TanstackPageContentDetail } from '@rainlanguage/ui-components';
  import {
    handleOrderRemoveModal,
    handleDebugTradeModal,
    handleQuoteDebugModal,
  } from '$lib/services/modal';
  import { createQuery } from '@tanstack/svelte-query';
  import { QKEY_ORDER } from '@rainlanguage/ui-components';
  import { OrderTradesListTable } from '@rainlanguage/ui-components';
  import { OrderTradesChart } from '@rainlanguage/ui-components';
  import { TanstackOrderQuote } from '@rainlanguage/ui-components';
  import { onDestroy } from 'svelte';
  import { queryClient } from '$lib/queries/queryClient';

  import { OrderVaultsVolTable } from '@rainlanguage/ui-components';
  import { colorTheme, lightweightChartsTheme, codeMirrorTheme } from '$lib/stores/darkMode';
  export let id, network;
  const subgraphUrl = $settings?.subgraphs?.[network] || '';
  const rpcUrl = $settings?.networks?.[network]?.rpc || '';
  import type { Order } from '@rainlanguage/orderbook/js_api';
  import { getOrder } from '@rainlanguage/orderbook/js_api';

  $: orderDetailQuery = createQuery<Order>({
    queryKey: [id, QKEY_ORDER + id],
    queryFn: () => getOrder(subgraphUrl || '', id),
    enabled: !!subgraphUrl,
  });

  const interval = setInterval(async () => {
    // This invalidate function invalidates
    // both order detail and order trades list queries
    await queryClient.invalidateQueries({
      queryKey: [id],
      refetchType: 'active',
      exact: false,
    });
  }, 10000);

  onDestroy(() => {
    clearInterval(interval);
  });
</script>

<TanstackPageContentDetail query={orderDetailQuery} emptyMessage="Order not found">
  <svelte:fragment slot="top" let:data={order}>
    <div class="flex gap-x-4 text-3xl font-medium dark:text-white">
      <div class="flex gap-x-2">
        <span class="font-light">Order</span>
        <Hash shorten value={order.orderHash} />
      </div>
      <BadgeActive active={order.active} large />
    </div>
    {#if order && $walletAddressMatchesOrBlank(order.owner) && order.active}
      <Button
        color="dark"
        on:click={() => handleOrderRemoveModal(order, $orderDetailQuery.refetch)}
      >
        Remove
      </Button>
    {/if}
  </svelte:fragment>
  <svelte:fragment slot="card" let:data={order}>
    <div class="flex flex-col gap-y-6">
      <CardProperty>
        <svelte:fragment slot="key">Orderbook</svelte:fragment>
        <svelte:fragment slot="value">
          <Hash type={HashType.Identifier} shorten={false} value={order.orderbook.id} />
        </svelte:fragment>
      </CardProperty>

      <CardProperty>
        <svelte:fragment slot="key">Owner</svelte:fragment>
        <svelte:fragment slot="value">
          <Hash type={HashType.Wallet} shorten={false} value={order.owner} />
        </svelte:fragment>
      </CardProperty>

      <CardProperty>
        <svelte:fragment slot="key">Created</svelte:fragment>
        <svelte:fragment slot="value">
          {formatTimestampSecondsAsLocal(BigInt(order.timestampAdded))}
        </svelte:fragment>
      </CardProperty>

      <CardProperty>
        <svelte:fragment slot="key">Input vaults</svelte:fragment>
        <svelte:fragment slot="value">
          <div class="mb-2 flex justify-end">
            <span>Balance</span>
          </div>
          <div class="space-y-2">
            {#each order.inputs || [] as t}
              <ButtonVaultLink tokenVault={t} />
            {/each}
          </div>
        </svelte:fragment>
      </CardProperty>

      <CardProperty>
        <svelte:fragment slot="key">Output vaults</svelte:fragment>
        <svelte:fragment slot="value">
          <div class="mb-2 flex justify-end">
            <span>Balance</span>
          </div>
          <div class="space-y-2">
            {#each order.outputs || [] as t}
              <ButtonVaultLink tokenVault={t} />
            {/each}
          </div>
        </svelte:fragment>
      </CardProperty>
    </div>
  </svelte:fragment>
  <svelte:fragment slot="chart">
    <OrderTradesChart {id} {subgraphUrl} {colorTheme} {lightweightChartsTheme} />
  </svelte:fragment>
  <svelte:fragment slot="below" let:data={order}>
    <TanstackOrderQuote
      {id}
      {order}
      {rpcUrl}
      orderbookAddress={$orderbookAddress}
      {handleQuoteDebugModal}
    />
    <Tabs
      style="underline"
      contentClass="mt-4"
      defaultClass="flex flex-wrap space-x-2 rtl:space-x-reverse mt-4"
    >
      <TabItem open title="Rainlang source">
        <div class="mb-8 overflow-hidden rounded-lg border dark:border-none">
          <CodeMirrorRainlang disabled={true} {order} {codeMirrorTheme} />
        </div>
      </TabItem>
      <TabItem title="Trades">
        <OrderTradesListTable {id} {subgraphUrl} {rpcUrl} {handleDebugTradeModal} />
      </TabItem>
      <TabItem title="Volume">
        <OrderVaultsVolTable {id} {subgraphUrl} />
      </TabItem>
    </Tabs>
  </svelte:fragment>
</TanstackPageContentDetail>
