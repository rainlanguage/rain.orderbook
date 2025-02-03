<script lang="ts">
	import { AccordionItem, Input } from 'flowbite-svelte';

	import {
		DotrainOrderGui,
		type GuiFieldDefinition,
		type GuiPreset
	} from '@rainlanguage/orderbook/js_api';
	import ButtonSelectOption from './ButtonSelectOption.svelte';
	import DeploymentSectionHeader from './DeploymentSectionHeader.svelte';

	export let fieldDefinition: GuiFieldDefinition;
	export let gui: DotrainOrderGui;

	let currentValue: GuiPreset | undefined;
	let inputValue: string | null = null;

	$: if (gui) {
		try {
			currentValue = gui.getFieldValue(fieldDefinition.binding);
		} catch {
			currentValue = undefined;
		}
	}

	async function handlePresetClick(preset: GuiPreset) {
		inputValue = preset.value;
		gui?.saveFieldValue(fieldDefinition.binding, {
			isPreset: true,
			value: preset.id
		});
		currentValue = gui.getFieldValue(fieldDefinition.binding);
		await gui.getAllFieldValues();
		await gui.getCurrentDeployment();
	}

	async function handleCustomInputChange(value: string) {
		inputValue = value;
		gui?.saveFieldValue(fieldDefinition.binding, {
			isPreset: false,
			value: value
		});
		currentValue = gui.getFieldValue(fieldDefinition.binding);
		await gui.getAllFieldValues();
		await gui.getCurrentDeployment();
	}

	export let open = true;
</script>

<AccordionItem title={fieldDefinition.name} bind:open>
	<span slot="header" class="w-full">
		<DeploymentSectionHeader
			title={fieldDefinition.name}
			description={fieldDefinition.description}
			bind:open
			value={currentValue?.name || currentValue?.value}
		/>
	</span>
	<div class="flex w-full max-w-2xl flex-col gap-6">
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
		{#if fieldDefinition.binding !== 'is-fast-exit'}
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
		{/if}
	</div>
</AccordionItem>
