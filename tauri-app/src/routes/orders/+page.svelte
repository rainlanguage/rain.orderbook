<script lang="ts">
  import { PageHeader } from '@rainlanguage/ui-components';
  import { onMount } from 'svelte';
  import { OrdersListTable } from '@rainlanguage/ui-components';
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
  import { writable } from 'svelte/store';

  $: currentRoute = $page.url.pathname;

  onMount(async () => {
    if (!$activeOrderbook) {
      await resetActiveNetworkRef();
      resetActiveOrderbookRef();
    }
  });
</script>

<PageHeader title="Orders" pathname={$page.url.pathname} />

<OrdersListTable
  {activeNetworkRef}
  {activeOrderbookRef}
  {handleOrderRemoveModal}
  {activeSubgraphs}
  {settings}
  {accounts}
  {activeAccountsItems}
  {activeOrderStatus}
  {orderHash}
  {hideZeroBalanceVaults}
  {currentRoute}
  showMyItemsOnly={writable(false)}
/>
