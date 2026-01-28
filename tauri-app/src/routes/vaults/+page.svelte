<script lang="ts">
  import { PageHeader, VaultsListTable } from '@rainlanguage/ui-components';
  import { page } from '$app/stores';

  import {
    orderHash,
    activeAccountsItems,
    selectedChainIds,
    showInactiveOrders,
    hideZeroBalanceVaults,
    hideInactiveOrdersVaults,
    activeTokens,
    activeOrderbookAddresses,
  } from '$lib/stores/settings';

  import { handleDepositModal, handleWithdrawModal } from '$lib/services/modal';
  import { writable } from 'svelte/store';
</script>

<PageHeader title="Vaults" pathname={$page.url.pathname} />

<VaultsListTable
  {orderHash}
  {activeAccountsItems}
  {selectedChainIds}
  {showInactiveOrders}
  {hideZeroBalanceVaults}
  {hideInactiveOrdersVaults}
  {handleDepositModal}
  {handleWithdrawModal}
  {activeTokens}
  {activeOrderbookAddresses}
  showMyItemsOnly={writable(false)}
/>
