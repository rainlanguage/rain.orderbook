<script lang="ts">
  import { invalidateTanstackQueries, PageHeader } from '@rainlanguage/ui-components';
  import { page } from '$app/stores';
  import { OrderDetail } from '@rainlanguage/ui-components';
  import { codeMirrorTheme, lightweightChartsTheme, colorTheme } from '$lib/stores/darkMode';
  import { settings } from '$lib/stores/settings';
  import {
    handleDebugTradeModal,
    handleQuoteDebugModal,
    handleDepositModal,
    handleWithdrawModal,
    handleOrderRemoveModal,
  } from '$lib/services/modal';
  import type { Hex } from 'viem';
  import type { SgOrder, SgVault } from '@rainlanguage/orderbook';
  import { useQueryClient } from '@tanstack/svelte-query';
  import { Button } from 'flowbite-svelte';

  const queryClient = useQueryClient();
  const { orderHash, network } = $page.params;

  let orderbookAddress: Hex | undefined;
  let subgraphUrl: string | undefined;
  let rpcUrls: string[] | undefined;

  if ($settings) {
    if ($settings.orderbook.orderbooks[network]) {
      orderbookAddress = $settings.orderbook.orderbooks[network].address as Hex;
    }

    if ($settings.orderbook.subgraphs[network]) {
      subgraphUrl = $settings.orderbook.subgraphs[network].url;
    }

    if ($settings.orderbook.networks[network]) {
      rpcUrls = $settings.orderbook.networks[network].rpcs;
    }
  }

  function onRemove(order: SgOrder) {
    handleOrderRemoveModal(order, () => {
      invalidateTanstackQueries(queryClient, [orderHash]);
    });
  }

  function onDeposit(vault: SgVault) {
    handleDepositModal(vault, () => {
      invalidateTanstackQueries(queryClient, [orderHash]);
    });
  }

  function onWithdraw(vault: SgVault) {
    handleWithdrawModal(vault, () => {
      invalidateTanstackQueries(queryClient, [orderHash]);
    });
  }
</script>

<PageHeader title="Order" pathname={$page.url.pathname} />

{#if rpcUrls && subgraphUrl && orderbookAddress}
  <div data-testid="order-detail">
    <OrderDetail
      {orderHash}
      {rpcUrls}
      {subgraphUrl}
      {colorTheme}
      {codeMirrorTheme}
      {lightweightChartsTheme}
      {handleQuoteDebugModal}
      {handleDebugTradeModal}
      {orderbookAddress}
      {onRemove}
      {onDeposit}
      {onWithdraw}
    />
  </div>
{:else}
  <div class="flex h-full flex-col items-center justify-center gap-4">
    <div class="flex flex-col items-center">
      <p class="mb-2 text-red-500">Failed to load order</p>
      <p class="mb-2">
        Missing the following items from settings for <b>{network}</b> network.
      </p>
      <ul class="flex list-none flex-col gap-1">
        {#if !rpcUrls || rpcUrls.length === 0}
          <li><span class="font-semibold">RPC URLs</span></li>
        {/if}
        {#if !subgraphUrl}
          <li><span class="font-semibold">Subgraph URL</span></li>
        {/if}
        {#if !orderbookAddress}
          <li><span class="font-semibold">Orderbook Address</span></li>
        {/if}
      </ul>
    </div>
    <Button href="/settings">Go to settings</Button>
  </div>
{/if}
