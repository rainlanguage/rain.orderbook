<script lang="ts">
	import { PageHeader, VaultsListTable } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { showMyItemsOnly, orderHash } from '$lib/stores/settings';

	const { activeAccountsItems, showInactiveOrders } = $page.data.stores;
</script>

<PageHeader title="Vaults" pathname={$page.url.pathname} />

<VaultsListTable {orderHash} {showMyItemsOnly} {activeAccountsItems} {showInactiveOrders} />
