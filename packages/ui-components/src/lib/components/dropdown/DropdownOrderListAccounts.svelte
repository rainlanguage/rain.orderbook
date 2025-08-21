<script lang="ts">
	import DropdownCheckbox from './DropdownCheckbox.svelte';
	import { getAccountsAsOptions } from '$lib/utils/configHelpers';
	import { useRaindexClient } from '$lib/hooks/useRaindexClient';

	export let activeAccountsItems: Record<string, string> = {};
	export let onChange: (accounts: Record<string, string>) => void;

	const raindexClient = useRaindexClient();

	$: accounts = raindexClient.getAllAccounts();
	$: options = getAccountsAsOptions(accounts.value);

	function handleChange(event: CustomEvent<Record<string, string>>) {
		const newAccounts = event.detail;
		activeAccountsItems = newAccounts;
		onChange(newAccounts);
	}
</script>

<div data-testid="accounts-dropdown">
	<DropdownCheckbox
		{options}
		value={activeAccountsItems}
		on:change={handleChange}
		label="Accounts"
		allLabel="All accounts"
		emptyMessage="No accounts added"
	/>
</div>
