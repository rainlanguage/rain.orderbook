<script lang="ts">
	import { Checkbox, Label } from 'flowbite-svelte';

	export let checked: boolean = false;
	export let onChange: (checked: boolean) => void;

	function handleHideZeroBalanceChange() {
		const newValue = !checked;
		checked = newValue;
		onChange(newValue);
	}
</script>

<div data-testid="zero-balance-vault-checkbox" class="flex items-center gap-x-2">
	<Label
		for="hide-empty-vaults"
		class="cursor-pointer whitespace-nowrap text-sm font-medium text-gray-900 dark:text-gray-300"
	>
		Hide empty vaults
	</Label>
	<Checkbox id="hide-empty-vaults" {checked} on:change={handleHideZeroBalanceChange} />
</div>
