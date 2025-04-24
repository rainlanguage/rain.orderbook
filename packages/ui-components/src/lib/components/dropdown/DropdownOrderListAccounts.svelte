<script lang="ts">
	import type { AccountCfg } from '@rainlanguage/orderbook';
	import DropdownCheckbox from './DropdownCheckbox.svelte';
	import type { Writable, Readable } from 'svelte/store';
	export let accounts: Readable<Record<string, AccountCfg>> | undefined;
	export let activeAccountsItems: Writable<Record<string, string>> | undefined;
	$: options = Object.fromEntries(
		Object.entries($accounts ?? {}).map(([key, value]) => [key, value.address])
	);
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
