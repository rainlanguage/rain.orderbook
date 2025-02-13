<script lang="ts">
  import { PageHeader, TransactionStatus } from '@rainlanguage/ui-components';
  import { page } from '$app/stores';
  import { OrderDetail, transactionStore } from '@rainlanguage/ui-components';
  import { codeMirrorTheme, lightweightChartsTheme, colorTheme } from '$lib/stores/darkMode';
  import { settings } from '$lib/stores/settings';
  import { handleDebugTradeModal, handleQuoteDebugModal } from '$lib/services/modal';
  import { useQueryClient } from '@tanstack/svelte-query';

  const { id, network } = $page.params;

  const orderbookAddress = $settings?.orderbooks?.[network]?.address;
  const subgraphUrl = $settings?.subgraphs?.[network];
  const rpcUrl = $settings?.networks?.[network]?.rpc;
  const chainId = $settings?.networks?.[network]?.['chain-id'];
  const queryClient = useQueryClient();
  let toastOpen: boolean = false;
  let counter: number = 5;

  function triggerToast() {
    toastOpen = true;
    counter = 5;
    timeout();
  }

  function timeout() {
    if (--counter > 0) return setTimeout(timeout, 1000);
    toastOpen = false;
  }

  $: if ($transactionStore.status === TransactionStatus.SUCCESS) {
    queryClient.invalidateQueries({
      queryKey: [$page.params.id],
    });
    triggerToast();
  }
</script>

<PageHeader title="Order" pathname={$page.url.pathname} />
{#if rpcUrl && subgraphUrl && orderbookAddress}
  <OrderDetail
    {id}
    {rpcUrl}
    {subgraphUrl}
    {colorTheme}
    {codeMirrorTheme}
    {lightweightChartsTheme}
    {handleQuoteDebugModal}
    {handleDebugTradeModal}
    {orderbookAddress}
    {chainId}
  />
{/if}
