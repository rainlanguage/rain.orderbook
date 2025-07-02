<script lang="ts">
  import { PageHeader, VaultsListTable } from '@rainlanguage/ui-components';
  import { page } from '$app/stores';

  import {
    orderHash,
    accounts,
    activeAccountsItems,
    selectedChainIds,
    settings,
    showInactiveOrders,
    hideZeroBalanceVaults,
    activeAccounts,
  } from '$lib/stores/settings';

  import {
    handleDepositGenericModal,
    handleDepositModal,
    handleWithdrawModal,
  } from '$lib/services/modal';
  import { writable } from 'svelte/store';
</script>

<PageHeader title="Vaults" pathname={$page.url.pathname} />

<VaultsListTable
  {activeAccounts}
  {orderHash}
  {accounts}
  {activeAccountsItems}
  {selectedChainIds}
  {settings}
  {showInactiveOrders}
  {hideZeroBalanceVaults}
  {handleDepositGenericModal}
  {handleDepositModal}
  {handleWithdrawModal}
  showMyItemsOnly={writable(false)}
/>
