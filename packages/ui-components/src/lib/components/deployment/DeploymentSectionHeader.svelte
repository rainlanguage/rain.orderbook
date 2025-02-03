<script lang="ts">
	import { fade } from 'svelte/transition';

	export let title: string;
	export let description: string = '';
	export let open: boolean;
	export let value: string | undefined;
</script>

<div class="flex w-full items-center justify-between gap-12 pr-12">
	<div class="max-w-2xl flex-grow text-left">
		<h1 class="break-words text-2xl font-bold text-gray-900 lg:text-3xl dark:text-white">
			{title}
		</h1>
		<p class="break-words text-lg text-gray-600 lg:text-xl dark:text-gray-400">
			{description}
		</p>
	</div>

	{#if !open && value}
		<span in:fade class="text-2xl font-semibold">
			{value}
		</span>
	{/if}
</div>
