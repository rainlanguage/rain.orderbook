<script lang="ts">
	import { DropdownCheckbox } from '@rainlanguage/ui-components';
	import type { Writable } from 'svelte/store';
	export let accounts: Writable<Record<string, string>>;
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
