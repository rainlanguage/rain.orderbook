<script lang="ts">
	import { Button, Spinner } from 'flowbite-svelte';

	export let loading = false;
	export let disabled = false;
</script>

<Button disabled={loading || disabled} {...$$props} on:click>
	{#if loading}
		<Spinner class="mr-2 h-4 w-4" color="white" />
	{/if}
	<slot />
</Button>
