<script lang="ts">
	import DropdownCheckbox from './DropdownCheckbox.svelte';
	import type { Writable, Readable } from 'svelte/store';
	export let accounts: Readable<Record<string, string>>;
	export let activeAccountsItems: Writable<Record<string, string>>;
	$: options = $accounts;
</script>

<DropdownCheckbox
	{options}
	bind:value={$activeAccountsItems}
	label="Accounts"
	allLabel="All accounts"
	emptyMessage="No accounts added"
/>
