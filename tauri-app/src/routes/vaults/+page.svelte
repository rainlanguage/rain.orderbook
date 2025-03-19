<script lang="ts">
  import { PageHeader, VaultsListTable } from '@rainlanguage/ui-components';
  import { onMount } from 'svelte';
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
  import { writable } from 'svelte/store';

  onMount(async () => {
    if (!$activeOrderbook) {
      await resetActiveNetworkRef();
      resetActiveOrderbookRef();
    }
  });

  $: currentRoute = $page.url.pathname;
</script>

<PageHeader title="Vaults" pathname={$page.url.pathname} />

<VaultsListTable
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
  {currentRoute}
  showMyItemsOnly={writable(false)}
  signerAddress={writable('')}
/>
