<script lang="ts">
	import { Button } from 'flowbite-svelte';

	export let active = false;
</script>

<Button on:click disabled={active} {...$$props}>
	<slot />
</Button>
