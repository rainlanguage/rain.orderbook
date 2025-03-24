<script lang="ts">
  import { walletAddressMatchesOrBlank } from '$lib/stores/wallets';
  import { PageHeader } from '@rainlanguage/ui-components';
  import { page } from '$app/stores';
  import { VaultDetail } from '@rainlanguage/ui-components';
  import { lightweightChartsTheme } from '$lib/stores/darkMode';
  import { handleDepositModal, handleWithdrawModal } from '$lib/services/modal';
  import { settings, activeNetworkRef, activeOrderbookRef } from '$lib/stores/settings';
  import type { SgVault } from '@rainlanguage/orderbook/js_api';
  import { useQueryClient } from '@tanstack/svelte-query';

  const queryClient = useQueryClient();

  // Handle deposit event
  function onDeposit(event: CustomEvent<{ vault: SgVault }>) {
    const { vault } = event.detail;

    // Use the Tauri deposit modal handler
    handleDepositModal(vault, () => {
      // Refresh data after deposit
      queryClient.invalidateQueries({
        queryKey: [$page.params.id],
        refetchType: 'all',
        exact: false,
      });
    });
  }

  // Handle withdraw event
  function onWithdraw(event: CustomEvent<{ vault: SgVault }>) {
    const { vault } = event.detail;

    // Use the Tauri withdraw modal handler
    handleWithdrawModal(vault, () => {
      // Refresh data after withdraw
      queryClient.invalidateQueries({
        queryKey: [$page.params.id],
        refetchType: 'all',
        exact: false,
      });
    });
  }
</script>

<PageHeader title="Vault" pathname={$page.url.pathname} />

<VaultDetail
  id={$page.params.id}
  network={$page.params.network}
  {lightweightChartsTheme}
  {settings}
  {walletAddressMatchesOrBlank}
  {activeNetworkRef}
  {activeOrderbookRef}
  on:deposit={onDeposit}
  on:withdraw={onWithdraw}
/>
