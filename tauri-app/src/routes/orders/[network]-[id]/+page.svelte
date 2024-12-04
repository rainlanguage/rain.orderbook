<script lang="ts">
  import { PageHeader } from '@rainlanguage/ui-components';
  import { page } from '$app/stores';
  import { OrderDetail } from '@rainlanguage/ui-components';
  import { codeMirrorTheme, lightweightChartsTheme, colorTheme } from '$lib/stores/darkMode';
  import { handleDebugTradeModal, handleQuoteDebugModal } from '$lib/services/modal';
  import { subgraphUrl, rpcUrl, orderbookAddress } from '$lib/stores/settings';

  const { id, network } = $page.params;
</script>

<PageHeader title="Order" pathname={$page.url.pathname} />

{#if $rpcUrl && $subgraphUrl && $orderbookAddress}
  <OrderDetail
    {id}
    {network}
    rpcUrl={$rpcUrl}
    subgraphUrl={$subgraphUrl}
    {colorTheme}
    {codeMirrorTheme}
    {lightweightChartsTheme}
    {handleQuoteDebugModal}
    {handleDebugTradeModal}
    orderbookAddress={$orderbookAddress}
  />
{/if}
