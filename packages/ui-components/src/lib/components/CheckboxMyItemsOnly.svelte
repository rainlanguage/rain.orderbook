<script lang="ts">
	import { Checkbox, Label } from 'flowbite-svelte';
	import type { Writable } from 'svelte/store';

	export let showMyItemsOnly: Writable<boolean>;
	export let context: 'orders' | 'vaults';
	export let signerAddress: Writable<string | null> | undefined = undefined;

	function handleShowMyItemsChange() {
		$showMyItemsOnly = !$showMyItemsOnly;
	}
</script>

<div data-testid="show-my-items-checkbox" class="flex items-center gap-x-2">
	<Label
		for="show-my-items"
		class="cursor-pointer whitespace-nowrap text-sm font-medium text-gray-900 dark:text-gray-300"
	>
		Only show my {context}
	</Label>
	<Checkbox
		id="show-my-items"
		checked={$showMyItemsOnly}
		on:change={handleShowMyItemsChange}
		disabled={!$signerAddress}
	/>
</div>
