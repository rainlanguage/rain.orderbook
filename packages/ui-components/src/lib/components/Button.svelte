<script lang="ts">
	import { Button } from 'flowbite-svelte';

	export let customClass: string = '';
	export let defaultClass = 'border-1 border-grey-300 rounded-md';
</script>

<Button {...$$restProps} class={`${defaultClass} ${customClass}`} on:click>
	<slot />
</Button>
