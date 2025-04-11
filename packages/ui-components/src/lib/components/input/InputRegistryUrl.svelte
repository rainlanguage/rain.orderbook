<script lang="ts">
	import { Button, Input } from 'flowbite-svelte';
	import { useRegistry } from '../../providers/registry/useRegistry';
	import { loadRegistryUrl } from '$lib/services/loadRegistryUrl';

	const registry = useRegistry();
	let newRegistryUrl = $registry.getCurrentRegistry();
</script>

<div class="mb-4 flex w-full items-start gap-4" data-testid="registry-input">
	<Input
		id="strategy-url"
		type="url"
		placeholder="Enter URL to raw strategy registry file"
		bind:value={newRegistryUrl}
	/>
	<Button class="text-nowrap" on:click={() => loadRegistryUrl(newRegistryUrl, $registry)}
		>Load Registry URL</Button
	>
</div>
