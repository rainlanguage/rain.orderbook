<script lang="ts">
	import { Button, Input } from 'flowbite-svelte';

	export let newRegistryUrl: string;

	const loadRegistryUrl = () => {
		window.history.pushState({}, '', window.location.pathname + '?registry=' + newRegistryUrl);
		window.location.reload();
		localStorage.setItem('registry', newRegistryUrl);
	};
</script>

<div class="flex w-full items-start gap-4">
	<Input
		id="strategy-url"
		type="url"
		placeholder="Enter URL to raw strategy registry file"
		bind:value={newRegistryUrl}
	/>
	<Button class="text-nowrap" on:click={loadRegistryUrl}>Load Registry URL</Button>
</div>
