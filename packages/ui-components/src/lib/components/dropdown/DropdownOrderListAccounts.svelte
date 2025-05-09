<script lang="ts">
	import DropdownCheckbox from './DropdownCheckbox.svelte';
	import type { AppStoresInterface } from '$lib/types/appStores';
	export let accounts: AppStoresInterface['accounts'];
	export let activeAccountsItems: AppStoresInterface['activeAccountsItems'];

	$: options = $accounts;
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
