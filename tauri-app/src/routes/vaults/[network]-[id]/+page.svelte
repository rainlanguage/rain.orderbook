<script lang="ts">
  import { invalidateTanstackQueries, PageHeader } from '@rainlanguage/ui-components';
  import { page } from '$app/stores';
  import { VaultDetail } from '@rainlanguage/ui-components';
  import { lightweightChartsTheme } from '$lib/stores/darkMode';
  import { handleDepositModal, handleWithdrawModal } from '$lib/services/modal';
  import { settings, activeNetworkRef, activeOrderbookRef } from '$lib/stores/settings';
  import type { SgVault } from '@rainlanguage/orderbook';
  import { useQueryClient } from '@tanstack/svelte-query';

  const queryClient = useQueryClient();

  function onDeposit(vault: SgVault) {
    handleDepositModal(vault, () => {
      invalidateTanstackQueries(queryClient, [$page.params.id]);
    });
  }

  function onWithdraw(vault: SgVault) {
    handleWithdrawModal(vault, () => {
      invalidateTanstackQueries(queryClient, [$page.params.id]);
    });
  }
</script>

<PageHeader title="Vault" pathname={$page.url.pathname} />

<VaultDetail
  id={$page.params.id}
  network={$page.params.network}
  {lightweightChartsTheme}
  {settings}
  {activeNetworkRef}
  {activeOrderbookRef}
  {onDeposit}
  {onWithdraw}
/>
