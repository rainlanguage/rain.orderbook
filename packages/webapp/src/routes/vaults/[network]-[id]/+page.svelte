<script lang="ts">
	import {
		VaultBalanceChangesTable,
		VaultBalanceChart,
		QKEY_VAULT
	} from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { createQuery } from '@tanstack/svelte-query';
	import { getVault } from '@rainlanguage/orderbook/js_api';

	const { settings } = $page.data.stores;
	const { lightweightChartsTheme } = $page.data.stores;
	const id = $page.params.id;
	const network = $page.params.network;
	const subgraphUrl = $settings?.subgraphs?.[network] || '';

	$: vaultDetailQuery = createQuery({
		queryKey: [id, QKEY_VAULT + id],
		queryFn: () => {
			return getVault(subgraphUrl || '', id);
		},
		enabled: !!subgraphUrl
	});
</script>

<VaultBalanceChart vault={$vaultDetailQuery.data} {subgraphUrl} {lightweightChartsTheme} />
<VaultBalanceChangesTable {id} {subgraphUrl} />
