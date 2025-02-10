<script lang="ts">
	import DropdownCheckbox from './DropdownCheckbox.svelte';
	import type { Writable, Readable } from 'svelte/store';
	export let accounts: Readable<Record<string, string>> | undefined;
	export let activeAccountsItems: Writable<Record<string, string>> | undefined;
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
