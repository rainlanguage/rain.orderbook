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
  } from '$lib/services/modal';
  import type { Hex } from 'viem';
  import type { SgVault } from '@rainlanguage/orderbook/js_api';
  import { Toast } from 'flowbite-svelte';
  import { CheckCircleSolid } from 'flowbite-svelte-icons';
  import { fade } from 'svelte/transition';
  import { useQueryClient } from '@tanstack/svelte-query';
  import { walletconnectAccount } from '$lib/stores/walletconnect';
  import { ledgerWalletAddress } from '$lib/stores/wallets';
  import { writable } from 'svelte/store';

  const queryClient = useQueryClient();
  const { orderHash, network } = $page.params;

  const orderbookAddress = $settings?.orderbooks?.[network]?.address as Hex;
  const subgraphUrl = $settings?.subgraphs?.[network];
  const rpcUrl = $settings?.networks?.[network]?.rpc;
  const chainId = $settings?.networks?.[network]?.['chain-id'];

  // Toast notification management
  let toastOpen: boolean = false;
  let toastMessage: string = 'Operation successful';
  let counter: number = 5;

  function triggerToast(message: string = 'Operation successful') {
    toastMessage = message;
    toastOpen = true;
    counter = 5;
    timeout();
  }

  function timeout() {
    if (--counter > 0) return setTimeout(timeout, 1000);
    toastOpen = false;
  }

  function invalidateOrderDetailQuery() {
    queryClient.invalidateQueries({
      queryKey: [orderHash],
      refetchType: 'all',
      exact: false,
    });
  }

  function onDeposit(vault: SgVault) {
    handleDepositModal(vault, () => {
      invalidateOrderDetailQuery();
      triggerToast('Deposit initiated');
    });
  }

  function onWithdraw(vault: SgVault) {
    handleWithdrawModal(vault, () => {
      invalidateOrderDetailQuery();
      triggerToast('Withdrawal initiated');
    });
  }
</script>

<PageHeader title="Order" pathname={$page.url.pathname} />

{#if toastOpen}
  <Toast dismissable={true} position="top-right" transition={fade}>
    <CheckCircleSolid slot="icon" class="h-5 w-5" />
    {toastMessage}
    <span class="text-sm text-gray-500">Autohide in {counter}s.</span>
  </Toast>
{/if}

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
    {onDeposit}
    {onWithdraw}
    signerAddress={$walletconnectAccount
      ? walletconnectAccount
      : $ledgerWalletAddress
        ? ledgerWalletAddress
        : writable(null)}
  />
{/if}
