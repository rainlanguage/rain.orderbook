<script lang="ts">
  import {
    invalidateTanstackQueries,
    PageHeader,
    useRaindexClient,
  } from '@rainlanguage/ui-components';
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
    VaultsList,
  } from '@rainlanguage/orderbook';
  import { useQueryClient } from '@tanstack/svelte-query';
  import { getAllContexts } from 'svelte';

  const context = getAllContexts();

  const raindexClient = useRaindexClient();
  const queryClient = useQueryClient();

  const { chainId, orderbook, orderHash } = $page.params;
  const parsedOrderHash = orderHash as Hex;
  const parsedChainId = Number(chainId);
  const orderbookAddress = orderbook as Address;

  let rpcs: string[] = [];

  $: if (raindexClient) {
    const networks = raindexClient.getNetworkByChainId(parsedChainId);
    if (networks.error) throw new Error(networks.error.readableMsg);
    rpcs = networks.value.rpcs;
  }

  function onRemove(_raindexClient: RaindexClient, order: RaindexOrder) {
    handleOrderRemoveModal(
      order,
      () => {
        invalidateTanstackQueries(queryClient, [parsedOrderHash]);
      },
      context,
    );
  }

  function onDeposit(_raindexClient: RaindexClient, vault: RaindexVault) {
    handleDepositModal(
      vault,
      () => {
        invalidateTanstackQueries(queryClient, [parsedOrderHash]);
      },
      context,
    );
  }

  function onWithdraw(_raindexClient: RaindexClient, vault: RaindexVault) {
    handleWithdrawModal(
      vault,
      () => {
        invalidateTanstackQueries(queryClient, [parsedOrderHash]);
      },
      context,
    );
  }

  function onWithdrawAll(_raindexClient: RaindexClient, _vaultsList: VaultsList) {
    // TODO
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
    {onWithdrawAll}
    {rpcs}
  />
</div>
