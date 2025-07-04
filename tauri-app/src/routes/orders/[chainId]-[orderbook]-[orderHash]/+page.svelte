<script lang="ts">
  import { invalidateTanstackQueries, PageHeader } from '@rainlanguage/ui-components';
  import { page } from '$app/stores';
  import { OrderDetail } from '@rainlanguage/ui-components';
  import { codeMirrorTheme, lightweightChartsTheme, colorTheme } from '$lib/stores/darkMode';
  import {
    handleDebugTradeModal,
    handleQuoteDebugModal,
    handleDepositModal,
    handleWithdrawModal,
    handleOrderRemoveModal,
  } from '$lib/services/modal';
  import type {
    Address,
    Hex,
    RaindexClient,
    RaindexOrder,
    RaindexVault,
  } from '@rainlanguage/orderbook';
  import { useQueryClient } from '@tanstack/svelte-query';

  const queryClient = useQueryClient();
  const { chainId, orderbook, orderHash } = $page.params;
  const parsedOrderHash = orderHash as Hex;
  const parsedChainId = Number(chainId);
  const orderbookAddress = orderbook as Address;

  function onRemove(_raindexClient: RaindexClient, order: RaindexOrder) {
    handleOrderRemoveModal(order, () => {
      invalidateTanstackQueries(queryClient, [parsedOrderHash]);
    });
  }

  function onDeposit(_raindexClient: RaindexClient, vault: RaindexVault) {
    handleDepositModal(vault, () => {
      invalidateTanstackQueries(queryClient, [parsedOrderHash]);
    });
  }

  function onWithdraw(_raindexClient: RaindexClient, vault: RaindexVault) {
    handleWithdrawModal(vault, () => {
      invalidateTanstackQueries(queryClient, [parsedOrderHash]);
    });
  }
</script>

<PageHeader title="Order" pathname={$page.url.pathname} />

<div data-testid="order-detail">
  <OrderDetail
    chainId={parsedChainId}
    {orderbookAddress}
    orderHash={parsedOrderHash}
    {colorTheme}
    {codeMirrorTheme}
    {lightweightChartsTheme}
    {handleQuoteDebugModal}
    {handleDebugTradeModal}
    {onRemove}
    {onDeposit}
    {onWithdraw}
  />
</div>
