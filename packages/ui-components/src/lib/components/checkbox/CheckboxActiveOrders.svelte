<script lang="ts">
	import { Checkbox } from 'flowbite-svelte';
	import type { Writable } from 'svelte/store';
	export let activeOrderStatus: Writable<boolean | undefined>;

	let includeInactive = $activeOrderStatus;
	$: activeOrderStatus.set(includeInactive ? undefined : true);
</script>

<div data-testid="order-status-checkbox">
	<Checkbox bind:checked={includeInactive}>Include inactive orders</Checkbox>
</div>
