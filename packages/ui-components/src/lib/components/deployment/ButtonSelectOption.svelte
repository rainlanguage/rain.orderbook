<script lang="ts">
	import { Button } from 'flowbite-svelte';
	export let active: boolean;
	export let buttonText: string;
	export let clickHandler: () => void;
	export let dataTestId: string;
</script>

<Button
	size="lg"
	color={active ? 'blue' : 'none'}
	class={active
		? 'flex-1 border border-gray-200 bg-white text-blue-600 shadow-sm hover:bg-white dark:border-gray-600 dark:bg-gray-800 dark:text-blue-400'
		: 'flex-1 border-none bg-transparent text-gray-600 hover:bg-gray-200/50 hover:text-gray-800 dark:text-gray-300 dark:hover:bg-gray-600/50 dark:hover:text-white'}
	on:click={clickHandler}
	data-testid={dataTestId}
>
	{buttonText}
</Button>
