<script lang="ts">
	import { Checkbox } from 'flowbite-svelte';

	export let checked: boolean = false;
	export let onChange: (checked: boolean) => void;

	function handleChange() {
		onChange(!checked);
	}
</script>

<div data-testid="order-status-checkbox">
	<Checkbox {checked} on:change={handleChange}>Include inactive orders</Checkbox>
</div>
