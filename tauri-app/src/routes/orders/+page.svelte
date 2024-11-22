<script lang="ts">
  import PageHeader from '$lib/components/PageHeader.svelte';

  import { onMount } from 'svelte';
  import { OrdersListTable } from '@rainlanguage/ui-components';
  import { walletAddressMatchesOrBlank } from '$lib/stores/wallets';
  import { handleOrderRemoveModal } from '$lib/services/modal';

  import {
    activeSubgraphs,
    settings,
    accounts,
    activeAccountsItems,
    activeOrderStatus,
    orderHash,
    hideZeroBalanceVaults,
    resetActiveNetworkRef,
    resetActiveOrderbookRef,
    activeOrderbook,
    activeNetworkRef,
    activeOrderbookRef,
  } from '$lib/stores/settings';
  import { page } from '$app/stores';

  $: currentRoute = $page.url.pathname;

  onMount(async () => {
    if (!$activeOrderbook) {
      await resetActiveNetworkRef();
      resetActiveOrderbookRef();
    }
  });
</script>

<PageHeader title="Orders" />

<OrdersListTable
  {activeNetworkRef}
  {activeOrderbookRef}
  {walletAddressMatchesOrBlank}
  {handleOrderRemoveModal}
  {activeSubgraphs}
  {settings}
  {accounts}
  {activeAccountsItems}
  {activeOrderStatus}
  {orderHash}
  {hideZeroBalanceVaults}
  {currentRoute}
/>
