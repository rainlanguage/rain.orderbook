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

  import {
    handleDepositModal,
    handleWithdrawModal,
    handleWithdrawMultipleModal,
  } from '$lib/services/modal';
  import { writable } from 'svelte/store';
  import type { RaindexClient, RaindexVault } from '@rainlanguage/orderbook';

  function onWithdrawMultiple(_raindexClient: RaindexClient, vaults: RaindexVault[]) {
    handleWithdrawMultipleModal(vaults, () => {});
  }
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
  {onWithdrawMultiple}
  showMyItemsOnly={writable(false)}
/>
