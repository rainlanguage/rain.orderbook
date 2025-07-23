<script lang="ts">
  import { invalidateTanstackQueries, PageHeader } from '@rainlanguage/ui-components';
  import { page } from '$app/stores';
  import { OrderDetail } from '@rainlanguage/ui-components';
  import { codeMirrorTheme, lightweightChartsTheme, colorTheme } from '$lib/stores/darkMode';
  import {
    handleQuoteDebugModal,
    handleDepositModal,
    handleWithdrawModal,
    handleOrderRemoveModal,
    handleDebugTradeModal,
  } from '$lib/services/modal';
  import type {
    Address,
    Hex,
    RaindexClient,
    RaindexOrder,
    RaindexVault,
  } from '@rainlanguage/orderbook';
  import { useQueryClient } from '@tanstack/svelte-query';
  import { getNetworkByChainId } from '$lib/utils/raindexClient/getNetworkByChainId';
  import { orderbookAddress as orderbookAddressStore } from '$lib/stores/settings';

  const queryClient = useQueryClient();
  const { chainId, orderbook, orderHash } = $page.params;
  const parsedOrderHash = orderHash as Hex;
  const parsedChainId = Number(chainId);
  const orderbookAddress = orderbook as Address;
  const network = getNetworkByChainId(parsedChainId);

  orderbookAddressStore.set(orderbookAddress);

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
    rpcUrls={network.rpcs}
  />
</div>
