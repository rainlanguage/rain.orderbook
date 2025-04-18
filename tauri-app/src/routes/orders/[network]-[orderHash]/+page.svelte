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

  const queryClient = useQueryClient();
  const { orderHash, network } = $page.params;

  const orderbookAddress = $settings?.orderbooks?.[network]?.address as Hex;
  const subgraphUrl = $settings?.subgraphs?.[network].url;
  const rpcUrl = $settings?.networks?.[network]?.rpc;

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

{#if rpcUrl && subgraphUrl && orderbookAddress}
  <OrderDetail
    {orderHash}
    {rpcUrl}
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
{/if}
