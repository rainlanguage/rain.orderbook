<script lang="ts">
	import { Checkbox } from 'flowbite-svelte';
	import type { Writable } from 'svelte/store';
	export let showInactiveOrders: Writable<boolean>;
	let checked: boolean = $showInactiveOrders ? true : false;

	function handleChange() {
		$showInactiveOrders = !$showInactiveOrders;
	}
</script>

<div data-testid="order-status-checkbox">
	<Checkbox {checked} on:change={handleChange}>Include inactive orders</Checkbox>
</div>
