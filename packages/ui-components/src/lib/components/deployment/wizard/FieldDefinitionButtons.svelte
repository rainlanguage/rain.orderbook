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
				<Button
					size="lg"
					color="alternative"
					class={gui?.isFieldPreset(fieldDefinition.binding) &&
					gui?.getFieldValue(fieldDefinition.binding)?.value === preset.id
						? 'border-2 border-gray-900 dark:border-white'
						: 'border border-gray-200 dark:border-gray-700'}
					on:click={() => handlePresetClick(preset.id)}
				>
					{preset.name || preset.value}
				</Button>
			{/each}
			{#if fieldDefinition.binding !== 'is-fast-exit'}
				<Button
					size="lg"
					color="alternative"
					class={!gui?.isFieldPreset(fieldDefinition.binding)
						? 'border-2 border-gray-900 dark:border-white'
						: 'border border-gray-200 dark:border-gray-700'}
					on:click={handleCustomClick}
				>
					Custom
				</Button>
			{/if}
		{/if}
	</div>
	{#if fieldDefinition.binding !== 'is-fast-exit'}
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
	{/if}
</div>
