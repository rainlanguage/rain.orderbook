<script lang="ts">
  import PageHeader from '$lib/components/PageHeader.svelte';
  import {
    activeOrderbook,
    resetActiveNetworkRef,
    resetActiveOrderbookRef,
  } from '$lib/stores/settings';
  import { onMount } from 'svelte';
  import VaultListTable from '$lib/components/tables/VaultListTable.svelte';
  import ModalVaultDeposit from '$lib/components/ModalVaultDeposit.svelte';
  import ModalVaultWithdraw from '$lib/components/ModalVaultWithdraw.svelte';
  import ModalVaultDepositGeneric from '$lib/components/ModalVaultDepositGeneric.svelte';
  import type { Vault } from '$lib/typeshare/vaultsList';

  onMount(async () => {
    if (!$activeOrderbook) {
      await resetActiveNetworkRef();
      resetActiveOrderbookRef();
    }
  });

  let showDepositModal = false;
  let showWithdrawModal = false;
  let showDepositGenericModal = false;
  let depositModalVault: Vault;
  let withdrawModalVault: Vault;

  const handleDepositGenericModal = () => {
    showDepositGenericModal = true;
  };

  const handleDepositModal = (e: CustomEvent<{ item: Vault }>) => {
    depositModalVault = e.detail.item;
    showDepositModal = true;
  };

  const handleWithdrawModal = (e: CustomEvent<{ item: Vault }>) => {
    withdrawModalVault = e.detail.item;
    showWithdrawModal = true;
  };
</script>

<PageHeader title="Vaults" />

<VaultListTable
  on:showDepositGenericModal={handleDepositGenericModal}
  on:showDepositModal={handleDepositModal}
  on:showWithdrawModal={handleWithdrawModal}
/>

<ModalVaultDeposit bind:open={showDepositModal} vault={depositModalVault} />
<ModalVaultWithdraw bind:open={showWithdrawModal} vault={withdrawModalVault} />
<ModalVaultDepositGeneric bind:open={showDepositGenericModal} />
