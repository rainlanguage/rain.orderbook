<script lang="ts">
	import { Button } from 'flowbite-svelte';
	import { fade } from 'svelte/transition';
	export let handleShareChoices: () => void;

	let showCopiedMessage = false;

	const handleClick = () => {
		handleShareChoices();
		showCopiedMessage = true;
		setTimeout(() => {
			showCopiedMessage = false;
		}, 2000);
	};
</script>

<div class="relative">
	{#if showCopiedMessage}
		<div
			transition:fade
			class="absolute -top-16 left-1/2 z-[200] -translate-x-1/2 whitespace-nowrap rounded-full border border-gray-200 bg-white px-8 py-4 text-sm dark:border-gray-600 dark:bg-gray-800 dark:text-white"
		>
			Shareable URL copied to clipboard
		</div>
	{/if}
	<Button color="light" size="lg" data-testid="review-choices-button" on:click={handleClick}
		>Share these choices</Button
	>
</div>
