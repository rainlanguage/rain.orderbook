<script lang="ts">
	import { Checkbox } from 'flowbite-svelte';
	import type { AppStoresInterface } from '$lib/types/appStores';
	export let showInactiveOrders: AppStoresInterface['showInactiveOrders'];
	let checked: boolean = $showInactiveOrders ? true : false;

	function handleChange() {
		$showInactiveOrders = !$showInactiveOrders;
	}
</script>

<div data-testid="order-status-checkbox">
	<Checkbox {checked} on:change={handleChange}>Include inactive orders</Checkbox>
</div>
