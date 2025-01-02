<script lang="ts">
	import {
		DotrainOrderGui,
		type GuiDeposit,
		type TokenInfos
	} from '@rainlanguage/orderbook/js_api';
	import { Input, Button } from 'flowbite-svelte';

	export let deposit: GuiDeposit;
	export let gui: DotrainOrderGui;
	export let tokenInfos: TokenInfos;

	let showCustomInput = !gui?.isDepositPreset(deposit.token_name);

	function handlePresetClick(preset: string) {
		gui?.saveDeposit(deposit.token_name, preset);
		showCustomInput = false;
		gui = gui;
	}

	function handleCustomClick() {
		showCustomInput = true;
		gui?.saveDeposit(deposit.token_name, '');
		gui = gui;
	}
</script>

<div class="flex flex-grow flex-col items-center p-8">
	<div class="mt-16 max-w-2xl text-center">
		<h1 class="mb-4 text-4xl font-bold text-gray-900 dark:text-white">
			{tokenInfos.get(deposit.token.address)?.name}
		</h1>
		<p class="mb-12 text-xl text-gray-600 dark:text-gray-400">Select deposit amount</p>
	</div>

	{#if deposit.presets}
		<div class="flex max-w-3xl flex-wrap justify-center gap-4">
			{#each deposit.presets as preset}
				<Button
					size="lg"
					color="alternative"
					class={gui?.isDepositPreset(deposit.token_name)
						? 'border border-gray-200 dark:border-gray-700'
						: 'border-2 border-gray-900 dark:border-white'}
					on:click={() => handlePresetClick(preset)}
				>
					{preset}
				</Button>
			{/each}
			<Button
				size="lg"
				color="alternative"
				class={!gui?.isDepositPreset(deposit.token_name)
					? 'border-2 border-gray-900 dark:border-white'
					: 'border border-gray-200 dark:border-gray-700'}
				on:click={handleCustomClick}
			>
				Custom
			</Button>
		</div>
	{/if}
	{#if showCustomInput}
		<div class="mt-8 w-full max-w-md">
			<Input
				class="text-center text-lg"
				size="lg"
				placeholder="Enter deposit amount"
				on:input={({ currentTarget }) => {
					if (currentTarget instanceof HTMLInputElement) {
						gui?.saveDeposit(deposit.token_name, currentTarget.value);
						gui = gui;
					}
				}}
			/>
		</div>
	{/if}
</div>
