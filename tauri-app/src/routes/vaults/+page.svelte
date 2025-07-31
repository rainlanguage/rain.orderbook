<script lang="ts">
  import { PageHeader, VaultsListTable } from '@rainlanguage/ui-components';
  import { page } from '$app/stores';

  import { orderHash, activeAccountsItems, showInactiveOrders } from '$lib/stores/settings';

  import { handleDepositModal, handleWithdrawModal } from '$lib/services/modal';
  import { writable } from 'svelte/store';
</script>

<PageHeader title="Vaults" pathname={$page.url.pathname} />

<VaultsListTable
  {orderHash}
  {activeAccountsItems}
  {showInactiveOrders}
  {handleDepositModal}
  {handleWithdrawModal}
  showMyItemsOnly={writable(false)}
/>
