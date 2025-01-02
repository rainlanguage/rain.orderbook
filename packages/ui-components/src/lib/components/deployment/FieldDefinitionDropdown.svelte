<script lang="ts">
	import { Label, Input } from 'flowbite-svelte';
	import DropdownRadio from '../dropdown/DropdownRadio.svelte';
	import { DotrainOrderGui, type GuiFieldDefinition } from '@rainlanguage/orderbook/js_api';

	export let fieldDefinition: GuiFieldDefinition;
	export let gui: DotrainOrderGui;
</script>

<div class="mb-4 flex flex-col gap-2">
	<Label class="whitespace-nowrap text-xl">{fieldDefinition.name}</Label>

	<DropdownRadio
		options={{
			...Object.fromEntries(
				(fieldDefinition.presets ?? []).map((preset) => [
					preset.id,
					{
						label: preset.name,
						id: preset.id
					}
				])
			),
			...{ custom: { label: 'Custom value', id: '' } }
		}}
		on:change={({ detail }) => {
			gui?.saveFieldValue(fieldDefinition.binding, {
				isPreset: detail.value !== 'custom',
				value: detail.value === 'custom' ? '' : detail.value || ''
			});
			gui = gui;
		}}
	>
		<svelte:fragment slot="content" let:selectedOption let:selectedRef>
			{#if selectedRef === undefined}
				<span>Select a preset</span>
			{:else if selectedOption?.label}
				<span>{selectedOption.label}</span>
			{:else}
				<span>{selectedRef}</span>
			{/if}
		</svelte:fragment>

		<svelte:fragment slot="option" let:option let:ref>
			<div class="w-full overflow-hidden overflow-ellipsis">
				<div class="text-md break-word">{option.label ? option.label : ref}</div>
			</div>
		</svelte:fragment>
	</DropdownRadio>

	{#if gui?.isFieldPreset(fieldDefinition.binding) === false}
		<Input
			placeholder="Enter value"
			on:change={({ currentTarget }) => {
				if (currentTarget instanceof HTMLInputElement) {
					gui?.saveFieldValue(fieldDefinition.binding, {
						isPreset: false,
						value: currentTarget.value
					});
				}
			}}
		/>
	{/if}
</div>
