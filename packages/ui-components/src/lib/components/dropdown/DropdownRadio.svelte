<script lang="ts" generics="T">
	import { Button, Dropdown, Radio } from 'flowbite-svelte';
	import { ChevronDownSolid } from 'flowbite-svelte-icons';
	import { sortBy } from 'lodash';
	import { createEventDispatcher } from 'svelte';

	const dispatch = createEventDispatcher<{
		change: { value: string | undefined };
	}>();

	// eslint-disable-next-line no-undef
	export let options: Record<string, T> = {};
	export let value: string | undefined = undefined;
	let open = false;

	$: {
		dispatch('change', { value });
		if (value) {
			open = false;
		}
	}
	$: optionsSorted = sortBy(Object.entries(options), (o) => o[0]);
</script>

<Button
	color="alternative"
	class="flex w-full justify-between overflow-hidden overflow-ellipsis pl-2 pr-0 text-left"
	data-testid="dropdown-button"
>
	<div class="flex-grow overflow-hidden">
		<slot
			name="content"
			selectedRef={value}
			selectedOption={value !== undefined ? options[value] : undefined}
		/>
	</div>
	<ChevronDownSolid class="mx-2 h-3 w-3 text-black dark:text-white" />
</Button>

<Dropdown bind:open data-testid="dropdown">
	{#each optionsSorted as [ref, option]}
		<Radio
			bind:group={value}
			value={ref}
			class="w-full rounded-lg p-3 hover:bg-gray-100 dark:hover:bg-gray-600"
		>
			<div class="ml-2">
				<slot name="option" {option} {ref} />
			</div>
		</Radio>
	{/each}
</Dropdown>
