<script lang="ts">
	import { Checkbox } from 'flowbite-svelte';
	import type { Writable } from 'svelte/store';
	export let showInactiveOrders: Writable<boolean | undefined>;

	let includeInactive = $showInactiveOrders;
	$: showInactiveOrders.set(includeInactive ? undefined : true);
</script>

<div data-testid="order-status-checkbox">
	<Checkbox bind:checked={includeInactive}>Include inactive orders</Checkbox>
</div>
