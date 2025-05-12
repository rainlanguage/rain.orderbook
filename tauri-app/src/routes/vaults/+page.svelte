<script lang="ts">
  import { PageHeader, VaultsListTable } from '@rainlanguage/ui-components';
  import { onMount } from 'svelte';
  import { page } from '$app/stores';

  import {
    activeOrderbook,
    subgraphUrl,
    orderHash,
    accounts,
    activeAccountsItems,
    activeSubgraphs,
    settings,
    activeOrderStatus,
    hideZeroBalanceVaults,
    activeNetworkRef,
    activeOrderbookRef,
    activeAccounts,
    resetActiveNetworkRef,
    resetActiveOrderbookRef,
  } from '$lib/stores/settings';

  import {
    handleDepositGenericModal,
    handleDepositModal,
    handleWithdrawModal,
  } from '$lib/services/modal';
  import { writable } from 'svelte/store';
  import { invalidateTanstackQueries } from '@rainlanguage/ui-components';
  import { useQueryClient } from '@tanstack/svelte-query';
  import type { SgVault } from '@rainlanguage/orderbook';
  const queryClient = useQueryClient();

  onMount(async () => {
    if (!$activeOrderbook) {
      await resetActiveNetworkRef();
      resetActiveOrderbookRef();
    }
  });

  function onDeposit(vault: SgVault) {
    handleDepositModal(vault, () => invalidateTanstackQueries(queryClient, [vault.id]));
  }

  function onWithdraw(vault: SgVault) {
    handleWithdrawModal(vault, () => invalidateTanstackQueries(queryClient, [vault.id]));
  }
</script>

<PageHeader title="Vaults" pathname={$page.url.pathname} />

<VaultsListTable
  {activeAccounts}
  {activeOrderbook}
  {subgraphUrl}
  {orderHash}
  {accounts}
  {activeAccountsItems}
  {activeSubgraphs}
  {settings}
  {activeOrderStatus}
  {hideZeroBalanceVaults}
  {activeNetworkRef}
  {activeOrderbookRef}
  {handleDepositGenericModal}
  {onDeposit}
  {onWithdraw}
  showMyItemsOnly={writable(false)}
/>
