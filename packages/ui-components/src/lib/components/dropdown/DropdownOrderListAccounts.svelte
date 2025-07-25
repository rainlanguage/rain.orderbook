<script lang="ts">
	import DropdownCheckbox from './DropdownCheckbox.svelte';
	import type { AppStoresInterface } from '$lib/types/appStores';
	import { getAccountsAsOptions } from '$lib/utils/configHelpers';
	import { useRaindexClient } from '$lib/hooks/useRaindexClient';

	export let activeAccountsItems: AppStoresInterface['activeAccountsItems'];

	const raindexClient = useRaindexClient();

	$: accounts = raindexClient.getAllAccounts();
	$: options = getAccountsAsOptions(accounts.value);
</script>

<div data-testid="accounts-dropdown">
	<DropdownCheckbox
		{options}
		bind:value={$activeAccountsItems}
		label="Accounts"
		allLabel="All accounts"
		emptyMessage="No accounts added"
	/>
</div>
