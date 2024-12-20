<script lang="ts">
	import { Label, Input, Button } from 'flowbite-svelte';
	import { DotrainOrderGui, type GuiFieldDefinition } from '@rainlanguage/orderbook/js_api';

	export let fieldDefinition: GuiFieldDefinition;
	export let gui: DotrainOrderGui;

	let showCustomInput: boolean = false;

	function handlePresetClick(presetId: string) {
		gui?.saveFieldValue(fieldDefinition.binding, {
			isPreset: true,
			value: presetId
		});
		showCustomInput = false;
		gui = gui;
	}

	function handleCustomClick() {
		showCustomInput = true;
		gui?.saveFieldValue(fieldDefinition.binding, {
			isPreset: false,
			value: ''
		});
		gui = gui;
	}

	console.log(fieldDefinition);
</script>

<div class="mb-4 flex flex-col gap-2">
	<Label class="whitespace-nowrap text-xl">{fieldDefinition.name}</Label>

	<div class="flex flex-wrap gap-2">
		{#if fieldDefinition.presets}
			{#each fieldDefinition.presets as preset}
				<Button
					color={gui?.isFieldPreset(fieldDefinition.binding) &&
					gui?.getFieldValue(fieldDefinition.binding)?.value === preset.id
						? 'blue'
						: 'alternative'}
					size="sm"
					on:click={() => handlePresetClick(preset.id)}
				>
					{preset.name}
				</Button>
			{/each}
		{/if}
		<Button
			color={!gui?.isFieldPreset(fieldDefinition.binding) ? 'blue' : 'alternative'}
			size="sm"
			on:click={handleCustomClick}
		>
			Custom
		</Button>
	</div>

	{#if !gui?.isFieldPreset(fieldDefinition.binding)}
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
