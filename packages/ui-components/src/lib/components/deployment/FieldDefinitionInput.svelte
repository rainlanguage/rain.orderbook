<script lang="ts">
	import { Input } from 'flowbite-svelte';

	import {
		DotrainOrderGui,
		type GuiFieldDefinitionCfg,
		type GuiPresetCfg
	} from '@rainlanguage/orderbook/js_api';
	import ButtonSelectOption from './ButtonSelectOption.svelte';
	import DeploymentSectionHeader from './DeploymentSectionHeader.svelte';
	import { onMount } from 'svelte';

	export let fieldDefinition: GuiFieldDefinitionCfg;
	export let gui: DotrainOrderGui;

	let currentValue: GuiPresetCfg | undefined;
	let inputValue: string | null = fieldDefinition.default
		? fieldDefinition.default
		: currentValue?.value || null;

	onMount(() => {
		try {
			currentValue = gui.getFieldValue(fieldDefinition.binding);
			inputValue = fieldDefinition.default ? fieldDefinition.default : currentValue?.value || null;
		} catch {
			currentValue = undefined;
		}
	});

	async function handlePresetClick(preset: GuiPresetCfg) {
		inputValue = preset.value;
		gui?.saveFieldValue(fieldDefinition.binding, {
			isPreset: true,
			value: preset.id
		});
		currentValue = gui.getFieldValue(fieldDefinition.binding);
	}

	async function handleCustomInputChange(value: string) {
		inputValue = value;
		gui?.saveFieldValue(fieldDefinition.binding, {
			isPreset: false,
			value: value
		});
		currentValue = gui.getFieldValue(fieldDefinition.binding);
	}
</script>

<div class="flex w-full flex-col gap-6">
	<DeploymentSectionHeader title={fieldDefinition.name} description={fieldDefinition.description} />
	<div class="flex w-full flex-col gap-6">
		{#if fieldDefinition.presets}
			<div class="flex w-full flex-wrap gap-4">
				{#each fieldDefinition.presets as preset}
					<ButtonSelectOption
						buttonText={preset.name || preset.value}
						clickHandler={() => handlePresetClick(preset)}
						active={currentValue?.value === preset.value}
					/>
				{/each}
			</div>
		{/if}
		<Input
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
</div>
