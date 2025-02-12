<script lang="ts">
	import { Progressbar } from 'flowbite-svelte';
	import { beforeNavigate, afterNavigate } from '$app/navigation';
	import { isNavigating } from '$lib/stores/loading';
	import { tick } from 'svelte';

	let progress = 0;
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	let interval: any;
	// Gradually increase progress while navigating
	const startLoading = () => {
		isNavigating.set(true);
		progress = 10; // Start at 10% so it's visible

		clearInterval(interval);
		interval = setInterval(() => {
			if (progress < 90) {
				progress += 5;
			}
		}, 300);
	};

	// Stop loading and reset progress
	const stopLoading = async () => {
		clearInterval(interval);
		progress = 100;
		await tick();
		setTimeout(() => {
			progress = 0; // Reset for next navigation
			isNavigating.set(false);
		}, 500);
	};

	beforeNavigate(startLoading);
	afterNavigate(stopLoading);
</script>

{#if $isNavigating}
	<div class="fixed left-0 top-0 z-50 w-full" data-testId="progressbar">
		<Progressbar {progress} color="blue" animate size="h-1" />
	</div>
{/if}

<slot />
