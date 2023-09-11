<script lang="ts">
	import { Tooltip } from 'flowbite-svelte';

	export let copyContent: string;

	const copy = async () => {
		await navigator.clipboard.writeText(copyContent);
	};
	const id = 'copy-button-' + Math.random().toString(36).substring(7);
</script>

<div on:click={copy}>
	<slot />
</div>
