<script lang="ts">
  import { walletAddressMatchesOrBlank } from '$lib/stores/wallets';
  import { PageHeader } from '@rainlanguage/ui-components';
  import { page } from '$app/stores';
  import { VaultDetail } from '@rainlanguage/ui-components';
  import { settings } from '$lib/stores/settings';
  import { lightweightChartsTheme } from '$lib/stores/darkMode';
  import { handleDepositModal, handleWithdrawModal } from '$lib/services/modal';
</script>

<PageHeader title="Vault" pathname={$page.url.pathname} />

<VaultDetail
  id={$page.params.id}
  network={$page.params.network}
  handleDespoitModal={handleDepositModal}
  {handleWithdrawModal}
  {lightweightChartsTheme}
  {settings}
  {walletAddressMatchesOrBlank}
/>
