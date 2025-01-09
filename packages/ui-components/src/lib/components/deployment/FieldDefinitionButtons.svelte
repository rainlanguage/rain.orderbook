<script lang="ts">
	import { Input } from 'flowbite-svelte';

	import {
		DotrainOrderGui,
		type GuiFieldDefinition,
		type GuiPreset
	} from '@rainlanguage/orderbook/js_api';
	import ButtonSelectOption from './ButtonSelectOption.svelte';

	export let fieldDefinition: GuiFieldDefinition;
	export let gui: DotrainOrderGui;

	$: currentFieldDefinition = gui?.getFieldValue(fieldDefinition.binding);
	let inputValue: string | null = null;

	function handlePresetClick(preset: GuiPreset) {
		inputValue = preset.value;
		gui?.saveFieldValue(fieldDefinition.binding, {
			isPreset: true,
			value: preset.id
		});
		gui = gui;
	}

	function handleCustomInputChange(value: string) {
		inputValue = value;
		gui?.saveFieldValue(fieldDefinition.binding, {
			isPreset: false,
			value: value
		});
		gui = gui;
	}

	$: if (fieldDefinition && !inputValue && inputValue !== '') {
		inputValue = currentFieldDefinition?.value || '';
	}
</script>

<div class="flex flex-grow flex-col items-center p-8">
	<!-- Header Section -->
	<div class="mt-16 max-w-2xl text-center">
		<h1 class="mb-4 text-4xl font-bold text-gray-900 dark:text-white">{fieldDefinition.name}</h1>
		<p class="mb-12 text-xl text-gray-600 dark:text-gray-400">
			{fieldDefinition.description}
		</p>
	</div>

	<!-- Buttons Section -->
	<div class="flex max-w-3xl flex-wrap justify-center gap-4">
		{#if fieldDefinition.presets}
			{#each fieldDefinition.presets as preset}
				<ButtonSelectOption
					buttonText={preset.name || preset.value}
					clickHandler={() => handlePresetClick(preset)}
					active={currentFieldDefinition?.value === preset.value}
				/>
			{/each}
		{/if}
	</div>
	{#if fieldDefinition.binding !== 'is-fast-exit'}
		<div class="mt-8 w-full max-w-md">
			<Input
				class="text-center text-lg"
				size="lg"
				placeholder="Enter custom value"
				bind:value={inputValue}
				on:input={({ currentTarget }) => {
					if (currentTarget instanceof HTMLInputElement) {
						handleCustomInputChange(currentTarget.value);
					}
				}}
			/>
		</div>
	{/if}
</div>
