<script lang="ts">
  import { ledgerWalletAddress } from '$lib/stores/wallets';
  import { PageHeader } from '@rainlanguage/ui-components';
  import { page } from '$app/stores';
  import { VaultDetail } from '@rainlanguage/ui-components';
  import { lightweightChartsTheme } from '$lib/stores/darkMode';
  import { handleDepositModal, handleWithdrawModal } from '$lib/services/modal';
  import { settings, activeNetworkRef, activeOrderbookRef } from '$lib/stores/settings';
  import type { SgVault } from '@rainlanguage/orderbook/js_api';
  import { useQueryClient } from '@tanstack/svelte-query';
  import { walletconnectAccount } from '$lib/stores/walletconnect';

  const queryClient = useQueryClient();

  function onDeposit(event: CustomEvent<{ vault: SgVault }>) {
    const { vault } = event.detail;
    handleDepositModal(vault, () => {
      queryClient.invalidateQueries({
        queryKey: [$page.params.id],
        refetchType: 'all',
        exact: false,
      });
    });
  }

  function onWithdraw(event: CustomEvent<{ vault: SgVault }>) {
    const { vault } = event.detail;
    handleWithdrawModal(vault, () => {
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
  signerAddress={$ledgerWalletAddress || $walletconnectAccount || ''}
  {activeNetworkRef}
  {activeOrderbookRef}
  on:deposit={onDeposit}
  on:withdraw={onWithdraw}
/>
