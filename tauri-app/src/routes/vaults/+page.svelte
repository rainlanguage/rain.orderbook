<script lang="ts">
  import { PageHeader } from '@rainlanguage/ui-components';
  import { onMount } from 'svelte';
  import VaultListTable from '$lib/components/tables/VaultListTable.svelte';
  import { page } from '$app/stores';
  import { walletAddressMatchesOrBlank } from '$lib/stores/wallets';

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

  onMount(async () => {
    if (!$activeOrderbook) {
      await resetActiveNetworkRef();
      resetActiveOrderbookRef();
    }
  });
</script>

<PageHeader title="Vaults" pathname={$page.url.pathname} />

<VaultListTable
  {walletAddressMatchesOrBlank}
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
  {handleDepositModal}
  {handleWithdrawModal}
/>
