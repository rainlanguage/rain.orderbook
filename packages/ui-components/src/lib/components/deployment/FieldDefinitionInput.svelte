<script lang="ts">
	import { AccordionItem, Input } from 'flowbite-svelte';

	import {
		DotrainOrderGui,
		type GuiFieldDefinition,
		type GuiPreset
	} from '@rainlanguage/orderbook/js_api';
	import ButtonSelectOption from './ButtonSelectOption.svelte';
	import DeploymentSectionHeader from './DeploymentSectionHeader.svelte';
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	export let fieldDefinition: GuiFieldDefinition;
	export let gui: DotrainOrderGui;
	export let open: boolean = true;

	let currentValue: GuiPreset | undefined;
	let inputValue: string | null = currentValue?.value || null;

	onMount(() => {
		try {
			currentValue = gui.getFieldValue(fieldDefinition.binding);
			inputValue = currentValue?.value || null;
		} catch {
			currentValue = undefined;
		}
	});

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

	$: console.log('OPEN in input', open);
	$: if ($page.url.searchParams) {
		console.log('params changed');
		console.log($page.url.searchParams.get('review'));
	}
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
