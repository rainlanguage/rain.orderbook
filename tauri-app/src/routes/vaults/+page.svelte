<script lang="ts">
  import { PageHeader, VaultsListTable } from '@rainlanguage/ui-components';
  import { page } from '$app/stores';

  import { handleDepositModal, handleWithdrawModal } from '$lib/services/modal';
</script>

<PageHeader title="Vaults" pathname={$page.url.pathname} />

<VaultsListTable {handleDepositModal} {handleWithdrawModal} />
