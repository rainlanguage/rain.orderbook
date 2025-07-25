<script lang="ts">
  import { invalidateTanstackQueries, PageHeader } from '@rainlanguage/ui-components';
  import { page } from '$app/stores';
  import { VaultDetail } from '@rainlanguage/ui-components';
  import { lightweightChartsTheme } from '$lib/stores/darkMode';
  import { handleDepositModal, handleWithdrawModal } from '$lib/services/modal';
  import type { Address, Hex, RaindexClient, RaindexVault } from '@rainlanguage/orderbook';
  import { useQueryClient } from '@tanstack/svelte-query';
  import { getAllContexts } from 'svelte';

  const context = getAllContexts();

  const { chainId, orderbook, id } = $page.params;
  const parsedId = id as Hex;
  const parsedChainId = Number(chainId);
  const orderbookAddress = orderbook as Address;

  const queryClient = useQueryClient();

  function onDeposit(_raindexClient: RaindexClient, vault: RaindexVault) {
    handleDepositModal(
      vault,
      () => {
        invalidateTanstackQueries(queryClient, [$page.params.id]);
      },
      context,
    );
  }

  function onWithdraw(_raindexClient: RaindexClient, vault: RaindexVault) {
    handleWithdrawModal(
      vault,
      () => {
        invalidateTanstackQueries(queryClient, [$page.params.id]);
      },
      context,
    );
  }
</script>

<PageHeader title="Vault" pathname={$page.url.pathname} />

<VaultDetail
  chainId={parsedChainId}
  {orderbookAddress}
  id={parsedId}
  {lightweightChartsTheme}
  {onDeposit}
  {onWithdraw}
/>
