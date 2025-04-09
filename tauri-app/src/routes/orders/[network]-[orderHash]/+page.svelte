<script lang="ts">
  import { PageHeader } from '@rainlanguage/ui-components';
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
  import type { SgOrder, SgVault } from '@rainlanguage/orderbook/js_api';
  import { useQueryClient } from '@tanstack/svelte-query';

  const queryClient = useQueryClient();
  const { orderHash, network } = $page.params;

  const orderbookAddress = $settings?.orderbooks?.[network]?.address as Hex;
  const subgraphUrl = $settings?.subgraphs?.[network];
  const rpcUrl = $settings?.networks?.[network]?.rpc;
  const chainId = $settings?.networks?.[network]?.['chain-id'];

  function invalidateOrderDetailQuery() {
    queryClient.invalidateQueries({
      queryKey: [orderHash],
      refetchType: 'all',
      exact: false,
    });
  }

  function onRemove(order: SgOrder) {
    handleOrderRemoveModal(order, () => {
      invalidateOrderDetailQuery();
    });
  }

  function onDeposit(vault: SgVault) {
    handleDepositModal(vault, () => {
      invalidateOrderDetailQuery();
    });
  }

  function onWithdraw(vault: SgVault) {
    handleWithdrawModal(vault, () => {
      invalidateOrderDetailQuery();
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
    {chainId}
    {onRemove}
    {onDeposit}
    {onWithdraw}
  />
{/if}
