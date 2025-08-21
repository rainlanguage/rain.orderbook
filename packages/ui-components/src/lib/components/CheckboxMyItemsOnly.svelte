<script lang="ts">
	import { Checkbox, Label } from 'flowbite-svelte';
	import { useAccount } from '$lib/providers/wallet/useAccount';
	import type { Address } from '@rainlanguage/orderbook';

	export let currentOwners: Address[] | undefined = undefined;
	export let onChange: (checked: boolean) => void;
	export let context: 'orders' | 'vaults';

	const { account } = useAccount();

	$: checked = !!(
		currentOwners &&
		currentOwners.length > 0 &&
		$account &&
		currentOwners.includes($account)
	);

	function handleShowMyItemsChange() {
		onChange(!checked);
	}
</script>

<div data-testid="show-my-items-checkbox" class="flex items-center gap-x-2">
	<Label
		for="show-my-items"
		class="cursor-pointer whitespace-nowrap text-sm font-medium text-gray-900 dark:text-gray-300"
	>
		Only show my {context}
	</Label>
	<Checkbox id="show-my-items" {checked} on:change={handleShowMyItemsChange} disabled={!$account} />
</div>
