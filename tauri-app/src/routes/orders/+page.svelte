<script lang="ts">
  import { PageHeader, OrdersListTable } from '@rainlanguage/ui-components';
  import { onMount } from 'svelte';
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
  showMyItemsOnly={writable(false)}
/>
