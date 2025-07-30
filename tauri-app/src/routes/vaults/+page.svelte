<script lang="ts">
  import { PageHeader, VaultsListTable } from '@rainlanguage/ui-components';
  import { page } from '$app/stores';

  import {
    orderHash,
    activeAccountsItems,
    selectedChainIds,
    showInactiveOrders,
    hideZeroBalanceVaults,
    activeTokens,
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
  {handleDepositModal}
  {handleWithdrawModal}
  {activeTokens}
  showMyItemsOnly={writable(false)}
/>
