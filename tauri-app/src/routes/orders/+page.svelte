<script lang="ts">
  import { PageHeader } from '@rainlanguage/ui-components';
  import { OrdersListTable } from '@rainlanguage/ui-components';
  import { handleOrderRemoveModal } from '$lib/services/modal';

  import {
    selectedChainIds,
    accounts,
    activeAccountsItems,
    showInactiveOrders,
    orderHash,
    hideZeroBalanceVaults,
    activeTokens,
  } from '$lib/stores/settings';
  import { page } from '$app/stores';
  import { writable } from 'svelte/store';
</script>

<PageHeader title="Orders" pathname={$page.url.pathname} />

<OrdersListTable
  {handleOrderRemoveModal}
  {selectedChainIds}
  {accounts}
  {activeAccountsItems}
  {showInactiveOrders}
  {orderHash}
  {hideZeroBalanceVaults}
  {activeTokens}
  showMyItemsOnly={writable(false)}
/>
