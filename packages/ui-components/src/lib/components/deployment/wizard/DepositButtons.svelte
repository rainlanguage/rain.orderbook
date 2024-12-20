<script lang="ts">
	import {
		DotrainOrderGui,
		type GuiDeposit,
		type TokenInfos
	} from '@rainlanguage/orderbook/js_api';
	import { Label, Input, Button } from 'flowbite-svelte';

	export let deposit: GuiDeposit;
	export let gui: DotrainOrderGui;
	export let tokenInfos: TokenInfos;

	let showCustomInput = false;

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

<div class="flex flex-grow flex-col items-center">
	<div class="mt-16 max-w-2xl text-center">
		<h1 class="mb-4 text-4xl font-bold text-gray-900 dark:text-white">
			{tokenInfos.get(deposit.token.address)?.name}
		</h1>
		<p class="mb-12 text-xl text-gray-600">Select deposit amount</p>
	</div>

	{#if deposit.presets}
		<div class="flex max-w-3xl flex-wrap justify-center gap-4">
			{#each deposit.presets as preset}
				<Button
					size="lg"
					color={gui?.isDepositPreset(deposit.token_name) &&
					gui?.getDeposit(deposit.token_name) === preset
						? 'blue'
						: 'alternative'}
					on:click={() => handlePresetClick(preset)}
				>
					{preset}
				</Button>
			{/each}
			<Button
				size="lg"
				color={!gui?.isDepositPreset(deposit.token_name) ? 'blue' : 'alternative'}
				on:click={handleCustomClick}
			>
				Custom
			</Button>
		</div>
	{/if}

	{#if !gui?.isDepositPreset(deposit.token_name)}
		<div class="mt-8 w-full max-w-md">
			<Input
				class="text-center text-lg"
				size="lg"
				placeholder="Enter deposit amount"
				on:change={({ currentTarget }) => {
					if (currentTarget instanceof HTMLInputElement) {
						gui?.saveDeposit(deposit.token_name, currentTarget.value);
					}
				}}
			/>
		</div>
	{/if}
</div>
