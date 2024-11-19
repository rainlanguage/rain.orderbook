<script lang="ts">
	import { DropdownCheckbox } from '@rainlanguage/ui-components';
	export let accounts;
	export let activeAccountsItems;
	$: options = $accounts;
</script>

<DropdownCheckbox
	{options}
	bind:value={$activeAccountsItems}
	label="Accounts"
	allLabel="All accounts"
	emptyMessage="No accounts added"
/>
