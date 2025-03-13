<script lang="ts">
  import { walletAddressMatchesOrBlank } from '$lib/stores/wallets';
  import { PageHeader } from '@rainlanguage/ui-components';
  import { page } from '$app/stores';
  import { VaultDetail } from '@rainlanguage/ui-components';
  import { lightweightChartsTheme } from '$lib/stores/darkMode';
  import { handleDepositModal, handleWithdrawModal } from '$lib/services/modal';
  import { settings, activeNetworkRef, activeOrderbookRef } from '$lib/stores/settings';
  import { Button } from 'flowbite-svelte';
  import { ArrowDownOutline, ArrowUpOutline } from 'flowbite-svelte-icons';
  import { derived } from 'svelte/store';

  const isCurrentUserOwner = derived(
    walletAddressMatchesOrBlank,
    ($walletAddressMatchesOrBlank) => {
      return (owner: string) => $walletAddressMatchesOrBlank(owner);
    },
  );
</script>

<PageHeader title="Vault" pathname={$page.url.pathname} />

<VaultDetail
  id={$page.params.id}
  network={$page.params.network}
  {lightweightChartsTheme}
  {settings}
  {activeNetworkRef}
  {activeOrderbookRef}
  {isCurrentUserOwner}
>
  <svelte:fragment slot="action-buttons" let:data let:query>
    <Button
      data-testid="vaultDetailDepositButton"
      color="dark"
      on:click={() => handleDepositModal(data, query)}
      ><ArrowDownOutline size="xs" class="mr-2" />Deposit</Button
    >
    <Button
      data-testid="vaultDetailWithdrawButton"
      color="dark"
      on:click={() => handleWithdrawModal(data, query)}
      ><ArrowUpOutline size="xs" class="mr-2" />Withdraw</Button
    >
  </svelte:fragment>
</VaultDetail>
