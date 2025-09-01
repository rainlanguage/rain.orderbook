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
		? 'flex-1 bg-white text-blue-600 shadow-sm border border-gray-200 hover:bg-white dark:bg-gray-800 dark:text-blue-400 dark:border-gray-600' 
		: 'flex-1 text-gray-600 bg-transparent border-none hover:bg-gray-200/50 hover:text-gray-800 dark:text-gray-300 dark:hover:bg-gray-600/50 dark:hover:text-white'
	}
	on:click={clickHandler}
	data-testid={dataTestId}
>
	{buttonText}
</Button>
