<script lang="ts">
	import { ArrowLeftOutline, ArrowRightOutline } from 'flowbite-svelte-icons';
	import { createEventDispatcher } from 'svelte';
	import { Button, Spinner } from 'flowbite-svelte';

	const dispatch = createEventDispatcher();

	export let index: number = 1;
	export let loading = false;
</script>

<div class="flex items-center">
	{#if loading}
		<Spinner class="mr-4 h-5 w-5" color="white" />
	{/if}
	<Button
		class="border-1 rounded-none rounded-l-lg"
		disabled={loading || index == 1}
		on:click={() => dispatch('previous')}
		color="light"
	>
		<ArrowLeftOutline class="me-2 h-3.5 w-3.5" />
		Previous
	</Button>
	<div
		class="justify-center border border-gray-300 bg-white px-5 py-2.5 text-center text-sm font-medium dark:border-gray-600 dark:bg-gray-800 dark:text-white"
	>
		{index}
	</div>
	<Button
		class="border-1 rounded-none rounded-r-lg"
		disabled={loading}
		on:click={() => dispatch('next')}
		color="light"
	>
		Next
		<ArrowRightOutline class="ms-2 h-3.5 w-3.5" />
	</Button>
</div>
