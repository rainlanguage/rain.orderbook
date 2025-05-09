<script lang="ts">
	import { Checkbox } from 'flowbite-svelte';
	import type { Writable } from 'svelte/store';
	export let activeOrderStatus: Writable<boolean>;
	let checked: boolean = $activeOrderStatus ? false : true;

	function handleChange() {
		$activeOrderStatus = $activeOrderStatus ? false : true;
	}
</script>

<div data-testid="order-status-checkbox">
	<Checkbox {checked} on:change={handleChange}>Include inactive orders</Checkbox>
</div>
