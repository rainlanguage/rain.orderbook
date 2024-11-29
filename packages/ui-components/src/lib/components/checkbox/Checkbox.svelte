<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { Checkbox as FlowbiteCheckbox, Label } from 'flowbite-svelte';

	const dispatch = createEventDispatcher();

	export let label: string = '';
	export let checked: boolean = false;

	function handleChange() {
		checked = !checked;
		dispatch('change', checked);
	}
</script>

<div class="flex items-center">
	<FlowbiteCheckbox {checked} on:click={handleChange} data-testid="checkbox" />
	<Label class="ml-2">
		{label}
	</Label>
</div>
