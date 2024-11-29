<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { Button, Dropdown, Label, Checkbox } from 'flowbite-svelte';
	import { ChevronDownSolid } from 'flowbite-svelte-icons';
	import { isEmpty } from 'lodash';

	const dispatch = createEventDispatcher();

	export let options: Record<string, string> = {};
	export let value: Record<string, string> = {};
	export let label: string = 'Select items';
	export let allLabel: string = 'All items';
	export let showAllLabel: boolean = true;
	export let emptyMessage: string = 'No items available';
	export let onlyTitle: boolean = false;

	$: selectedCount = Object.keys(value).length;
	$: allSelected = selectedCount === Object.keys(options).length;
	$: buttonText =
		selectedCount === 0
			? 'Select items'
			: allSelected
				? allLabel
				: `${selectedCount} item${selectedCount > 1 ? 's' : ''}`;

	function updateValue(newValue: Record<string, string>) {
		value = newValue;
		dispatch('change', value);
	}

	function toggleAll() {
		updateValue(allSelected ? {} : { ...options });
	}

	function toggleItem(key: string) {
		const newValue = { ...value };
		if (key in newValue) {
			delete newValue[key];
		} else {
			newValue[key] = options[key];
		}
		updateValue(newValue);
	}
</script>

<Label>{label}</Label>
<div>
	<Button
		color="alternative"
		class="flex w-full justify-between overflow-hidden pl-2 pr-0 text-left"
		data-testid="dropdown-checkbox-button"
	>
		<div class="w-[90px] overflow-hidden text-ellipsis whitespace-nowrap">
			{buttonText}
		</div>
		<ChevronDownSolid class="mx-2 h-3 w-3 text-black dark:text-white" />
	</Button>

	<Dropdown class="w-full min-w-72 py-0" data-testid="dropdown-checkbox">
		{#if isEmpty(options)}
			<div class="ml-2 w-full rounded-lg p-3">{emptyMessage}</div>
		{:else if Object.keys(options).length > 1 && showAllLabel}
			<Checkbox
				data-testid="dropdown-checkbox-option"
				class="w-full rounded-lg p-3 hover:bg-gray-100 dark:hover:bg-gray-600"
				on:click={toggleAll}
				checked={allSelected}
			>
				<div class="ml-2">{allLabel}</div>
			</Checkbox>
		{/if}

		{#each Object.entries(options) as [key, optionValue]}
			<Checkbox
				data-testid="dropdown-checkbox-option"
				class="w-full rounded-lg p-3 hover:bg-gray-100 dark:hover:bg-gray-600"
				on:click={() => toggleItem(key)}
				checked={key in value}
			>
				<div class="ml-2">
					<div class="text-sm font-medium">{key}</div>
					{#if !onlyTitle}
						<div class="text-xs text-gray-500">{optionValue}</div>
					{/if}
				</div>
			</Checkbox>
		{/each}
	</Dropdown>
</div>
