<script lang="ts">
	import { Checkbox, Label } from 'flowbite-svelte';
	import type { AppStoresInterface } from '$lib/types/appStores';
	export let hideInactiveOrdersVaults: AppStoresInterface['hideInactiveOrdersVaults'];

	function handleHideInactiveOrdersVaultsChange() {
		$hideInactiveOrdersVaults = !$hideInactiveOrdersVaults;
	}
</script>

<div data-testid="inactive-orders-vault-checkbox" class="flex items-center gap-x-2">
	<Label
		for="hide-inactive-orders-vaults"
		class="cursor-pointer whitespace-nowrap text-sm font-medium text-gray-900 dark:text-gray-300"
	>
		Hide vaults without active orders
	</Label>
	<Checkbox
		id="hide-inactive-orders-vaults"
		checked={$hideInactiveOrdersVaults}
		on:change={handleHideInactiveOrdersVaultsChange}
	/>
</div>
