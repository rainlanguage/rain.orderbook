<script lang="ts">
	import { fade } from 'svelte/transition';

	export let title: string;
	export let description: string = '';
	export let open: boolean;
	export let value: string | undefined;
</script>

<div class="flex w-full items-center justify-between gap-12 pr-6">
	<div class="max-w-xl flex-grow text-left">
		<h1 class="break-words text-xl font-semibold text-gray-900 lg:text-2xl dark:text-white">
			{title}
		</h1>
		<p class="break-words text-sm font-light text-gray-600 lg:text-base dark:text-gray-400">
			{description}
		</p>
	</div>

	{#if !open && value}
		<span in:fade class="text-right text-base font-semibold lg:text-lg">
			{value}
		</span>
	{/if}
</div>
