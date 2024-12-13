<script lang="ts">
	import { VaultBalanceChangesTable, VaultBalanceChart } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';

	const { settings } = $page.data.stores;
	const id = $page.params.id;
	const network = $page.params.network;
	const subgraphUrl = $settings?.subgraphs?.[network] || '';
</script>

<VaultBalanceChart {id} {subgraphUrl} />
<VaultBalanceChangesTable {id} {subgraphUrl} />
