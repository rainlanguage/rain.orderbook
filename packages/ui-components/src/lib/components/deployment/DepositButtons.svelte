<script lang="ts">
	import {
		DotrainOrderGui,
		type GuiDeposit,
		type TokenInfos,
		type TokenDeposit
	} from '@rainlanguage/orderbook/js_api';
	import { Input } from 'flowbite-svelte';
	import ButtonSelectOption from './ButtonSelectOption.svelte';

	export let deposit: GuiDeposit;
	export let gui: DotrainOrderGui;
	export let tokenInfos: TokenInfos;

	let currentDeposit: TokenDeposit | undefined;

	let tokenName = '';
	let inputValue = '';

	function handlePresetClick(preset: string) {
		inputValue = preset;
		gui?.saveDeposit(deposit.token.key, preset);
		gui = gui;
		currentDeposit = gui?.getDeposits().find((d) => d.token === deposit.token.key);
	}

	function handleInput(e: Event) {
		if (e.currentTarget instanceof HTMLInputElement) {
			inputValue = e.currentTarget.value;
			gui?.saveDeposit(deposit.token.key, e.currentTarget.value);
			gui = gui;
			currentDeposit = gui?.getDeposits().find((d) => d.token === deposit.token.key);
		}
	}

	$: if (tokenInfos) {
		tokenName = tokenInfos.get(deposit.token.address)?.name || deposit.token.key;
	}
</script>

<div class="flex flex-grow flex-col items-center p-8">
	<div class="mt-16 max-w-2xl text-center">
		<h1 class="mb-4 text-4xl font-bold text-gray-900 dark:text-white">
			{tokenName}
		</h1>
		<p class="mb-12 text-xl text-gray-600 dark:text-gray-400">Select deposit amount</p>
	</div>

	{#if deposit.presets}
		<div class="flex max-w-3xl flex-wrap justify-center gap-4">
			{#each deposit.presets as preset}
				<ButtonSelectOption
					active={currentDeposit?.amount === preset}
					buttonText={preset}
					clickHandler={() => handlePresetClick(preset)}
				/>
			{/each}
		</div>
	{/if}

	<div class="mt-8 w-full max-w-md">
		<Input
			class="text-center text-lg"
			size="lg"
			placeholder="Enter deposit amount"
			bind:value={inputValue}
			on:input={(e) => handleInput(e)}
		/>
	</div>
</div>
