<script lang="ts">
	import { Button, Input } from 'flowbite-svelte';
	export let loadRegistryUrl: (url: string) => void;
	let newRegistryUrl = localStorage.getItem('registry') || '';
</script>

<div class="mb-4 flex w-full items-start gap-4" data-testid="registry-input">
	<Input
		id="strategy-url"
		type="url"
		placeholder="Enter URL to raw strategy registry file"
		bind:value={newRegistryUrl}
	/>
	<Button class="text-nowrap" on:click={() => loadRegistryUrl(newRegistryUrl)}
		>Load Registry URL</Button
	>
</div>
