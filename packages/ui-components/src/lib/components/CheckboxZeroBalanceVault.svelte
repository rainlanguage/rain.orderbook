<script lang="ts">
	import { Checkbox, Label } from 'flowbite-svelte';
	import type { AppStoresInterface } from '$lib/types/appStores';
	export let hideZeroBalanceVaults: AppStoresInterface['hideZeroBalanceVaults'];

	function handleHideZeroBalanceChange() {
		$hideZeroBalanceVaults = !$hideZeroBalanceVaults;
	}
</script>

<div data-testid="zero-balance-vault-checkbox" class="flex items-center gap-x-2">
	<Label
		for="hide-empty-vaults"
		class="cursor-pointer whitespace-nowrap text-sm font-medium text-gray-900 dark:text-gray-300"
	>
		Hide empty vaults
	</Label>
	<Checkbox
		id="hide-empty-vaults"
		checked={$hideZeroBalanceVaults}
		on:change={handleHideZeroBalanceChange}
	/>
</div>
