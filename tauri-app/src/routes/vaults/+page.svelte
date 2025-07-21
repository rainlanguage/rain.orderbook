<script lang="ts">
  import {
    invalidateTanstackQueries,
    PageHeader,
    QKEY_VAULTS,
    VaultsListTable,
  } from '@rainlanguage/ui-components';
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
  import { queryClient } from '$lib/queries/queryClient';

  function onWithdrawMultiple(_raindexClient: RaindexClient, vaults: RaindexVault[]) {
    return new Promise<boolean>((resolve) => {
      handleWithdrawMultipleModal(
        vaults,
        () => {
          // Invalidate all vault queries to refresh the data
          invalidateTanstackQueries(queryClient, [QKEY_VAULTS]);
          resolve(true);
        },
        () => {
          // Handle cancel action if needed
          resolve(false);
        },
      );
    });
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
