<script lang="ts">
	import { Input } from 'flowbite-svelte';

	import {
		DotrainOrderGui,
		type GuiFieldDefinition,
		type GuiPreset
	} from '@rainlanguage/orderbook/js_api';
	import ButtonSelectOption from './ButtonSelectOption.svelte';
	import DeploymentSectionHeader from './DeploymentSectionHeader.svelte';

	export let fieldDefinition: GuiFieldDefinition;
	export let gui: DotrainOrderGui;

	let currentFieldDefinition: GuiPreset | undefined;
	let inputValue = '';

	function handlePresetClick(preset: GuiPreset) {
		inputValue = preset.value;
		gui?.saveFieldValue(fieldDefinition.binding, {
			isPreset: true,
			value: preset.id
		});
		gui = gui;
		currentFieldDefinition = gui?.getFieldValue(fieldDefinition.binding);
	}

	function handleCustomInputChange(value: string) {
		inputValue = value;
		gui?.saveFieldValue(fieldDefinition.binding, {
			isPreset: false,
			value: value
		});
		gui = gui;
		currentFieldDefinition = gui?.getFieldValue(fieldDefinition.binding);
	}

	$: console.log(currentFieldDefinition);
</script>

<div class="flex flex-grow flex-col items-center p-8">
	<DeploymentSectionHeader title={fieldDefinition.name} description={fieldDefinition.description} />

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
