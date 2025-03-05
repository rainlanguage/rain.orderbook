<script lang="ts">
  import { walletAddressMatchesOrBlank } from '$lib/stores/wallets';
  import { PageHeader } from '@rainlanguage/ui-components';
  import { page } from '$app/stores';
  import { VaultDetail } from '@rainlanguage/ui-components';
  import { lightweightChartsTheme } from '$lib/stores/darkMode';
  import { handleDepositModal, handleWithdrawModal } from '$lib/services/modal';
  import { settings, activeNetworkRef, activeOrderbookRef } from '$lib/stores/settings';
</script>

<PageHeader title="Vault" pathname={$page.url.pathname} />

<VaultDetail
  id={$page.params.id}
  network={$page.params.network}
  {handleDepositModal}
  {handleWithdrawModal}
  {lightweightChartsTheme}
  {settings}
  {walletAddressMatchesOrBlank}
  {activeNetworkRef}
  {activeOrderbookRef}
/>
