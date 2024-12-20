<script lang="ts">
	import { Label, Input, Button } from 'flowbite-svelte';
	import { DotrainOrderGui, type GuiFieldDefinition } from '@rainlanguage/orderbook/js_api';

	export let fieldDefinition: GuiFieldDefinition;
	export let gui: DotrainOrderGui;

	let showCustomInput = false;

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
</script>

<div class="flex min-h-screen flex-col items-center bg-gray-50 p-8">
	<!-- Header Section -->
	<div class="mt-16 max-w-2xl text-center">
		<h1 class="mb-4 text-4xl font-bold text-gray-900">{fieldDefinition.name}</h1>
		<p class="mb-12 text-xl text-gray-600">
			{fieldDefinition.description}
		</p>
	</div>

	<!-- Buttons Section -->
	<div class="flex max-w-3xl flex-wrap justify-center gap-4">
		{#if fieldDefinition.presets}
			{#each fieldDefinition.presets as preset}
				<Button
					size="lg"
					color={gui?.isFieldPreset(fieldDefinition.binding) &&
					gui?.getFieldValue(fieldDefinition.binding)?.value === preset.id
						? 'blue'
						: 'alternative'}
					on:click={() => handlePresetClick(preset.id)}
				>
					{preset.name}
				</Button>
			{/each}
		{/if}
		<Button
			size="lg"
			color={!gui?.isFieldPreset(fieldDefinition.binding) ? 'blue' : 'alternative'}
			on:click={handleCustomClick}
		>
			Custom
		</Button>
	</div>

	<!-- Custom Input Section -->
	{#if !gui?.isFieldPreset(fieldDefinition.binding)}
		<div class="mt-8 w-full max-w-md">
			<Input
				class="text-center text-lg"
				size="lg"
				placeholder="Enter custom value"
				on:change={({ currentTarget }) => {
					if (currentTarget instanceof HTMLInputElement) {
						gui?.saveFieldValue(fieldDefinition.binding, {
							isPreset: false,
							value: currentTarget.value
						});
					}
				}}
			/>
		</div>
	{/if}
</div>
