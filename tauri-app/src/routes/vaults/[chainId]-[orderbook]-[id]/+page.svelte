<script lang="ts">
  import { invalidateTanstackQueries, PageHeader } from '@rainlanguage/ui-components';
  import { page } from '$app/stores';
  import { VaultDetail } from '@rainlanguage/ui-components';
  import { lightweightChartsTheme } from '$lib/stores/darkMode';
  import { handleDepositModal, handleWithdrawModal } from '$lib/services/modal';
  import { activeNetworkRef, activeOrderbookRef } from '$lib/stores/settings';
  import type { Address, RaindexClient, RaindexVault } from '@rainlanguage/orderbook';
  import { useQueryClient } from '@tanstack/svelte-query';

  const { chainId, orderbook, id } = $page.params;
  const parsedChainId = Number(chainId);
  const orderbookAddress = orderbook as Address;

  const queryClient = useQueryClient();

  function onDeposit(_raindexClient: RaindexClient, vault: RaindexVault) {
    handleDepositModal(vault, () => {
      invalidateTanstackQueries(queryClient, [$page.params.id]);
    });
  }

  function onWithdraw(_raindexClient: RaindexClient, vault: RaindexVault) {
    handleWithdrawModal(vault, () => {
      invalidateTanstackQueries(queryClient, [$page.params.id]);
    });
  }
</script>

<PageHeader title="Vault" pathname={$page.url.pathname} />

<VaultDetail
  chainId={parsedChainId}
  {orderbookAddress}
  {id}
  {lightweightChartsTheme}
  {activeNetworkRef}
  {activeOrderbookRef}
  {onDeposit}
  {onWithdraw}
/>
