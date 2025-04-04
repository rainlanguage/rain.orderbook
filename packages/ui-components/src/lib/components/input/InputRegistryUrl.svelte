<script lang="ts">
	import { Button, Input } from 'flowbite-svelte';

	let newRegistryUrl = localStorage.getItem('registry') || '';

	const loadRegistryUrl = () => {
		window.history.pushState({}, '', window.location.pathname + '?registry=' + newRegistryUrl);
		localStorage.setItem('registry', newRegistryUrl);
		window.location.reload();
	};
</script>

<div class="mb-4 flex w-full items-start gap-4" data-testid="registry-input">
	<Input
		id="strategy-url"
		type="url"
		placeholder="Enter URL to raw strategy registry file"
		bind:value={newRegistryUrl}
	/>
	<Button class="text-nowrap" on:click={loadRegistryUrl}>Load Registry URL</Button>
</div>
