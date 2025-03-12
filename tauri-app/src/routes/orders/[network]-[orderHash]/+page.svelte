<script lang="ts">
  import { PageHeader } from '@rainlanguage/ui-components';
  import { page } from '$app/stores';
  import { OrderDetail } from '@rainlanguage/ui-components';
  import { codeMirrorTheme, lightweightChartsTheme, colorTheme } from '$lib/stores/darkMode';
  import { settings } from '$lib/stores/settings';
  import {
    handleDebugTradeModal,
    handleQuoteDebugModal,
    handleOrderRemoveModal,
  } from '$lib/services/modal';
  import type { Hex } from 'viem';
  import { walletAddressMatchesOrBlank } from '$lib/stores/wallets';
  const { orderHash, network } = $page.params;

  const orderbookAddress = $settings?.orderbooks?.[network]?.address as Hex;
  const subgraphUrl = $settings?.subgraphs?.[network];
  const rpcUrl = $settings?.networks?.[network]?.rpc;
  const chainId = $settings?.networks?.[network]?.['chain-id'];
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
    {handleOrderRemoveModal}
    {walletAddressMatchesOrBlank}
    {orderbookAddress}
    {chainId}
  />
{/if}
