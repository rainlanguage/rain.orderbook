<script lang="ts">
	import { PageHeader } from '@rainlanguage/ui-components';
	import { onMount } from 'svelte';
	import VaultListTable from '$lib/components/tables/VaultListTable.svelte';
	import { page } from '$app/stores';

	const { activeOrderbook, resetActiveNetworkRef, resetActiveOrderbookRef } = $page.data.stores;

	onMount(async () => {
		if (!$activeOrderbook) {
			await resetActiveNetworkRef();
			resetActiveOrderbookRef();
		}
	});
</script>

<PageHeader title="Vaults" pathname={$page.url.pathname} />

<VaultListTable />
