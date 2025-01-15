<script lang="ts">
	import {
		DotrainOrderGui,
		type GuiDeposit,
		type TokenDeposit
	} from '@rainlanguage/orderbook/js_api';
	import { Input } from 'flowbite-svelte';
	import ButtonSelectOption from './ButtonSelectOption.svelte';
	import DeploymentSectionHeader from './DeploymentSectionHeader.svelte';

	export let deposit: GuiDeposit;
	export let gui: DotrainOrderGui;

	let currentDeposit: TokenDeposit | undefined;

	let tokenName: string = 'Deposit amount';
	let inputValue: string = '';

	$: console.log('curr deposit', currentDeposit);

	function handlePresetClick(preset: string) {
		if (deposit.token?.key) {
			inputValue = preset;
			gui?.saveDeposit(deposit.token?.key, preset);
			gui = gui;
			currentDeposit = gui?.getDeposits().find((d) => d.token === deposit.token?.key);
		}
	}

	function handleInput(e: Event) {
		if (deposit.token?.key) {
			if (e.currentTarget instanceof HTMLInputElement) {
				inputValue = e.currentTarget.value;
				gui?.saveDeposit(deposit.token.key, e.currentTarget.value);
				gui = gui;
				currentDeposit = gui?.getDeposits().find((d) => d.token === deposit.token?.key);
			}
		}
	}

	$: if (deposit) {
		if (deposit.token?.symbol) {
			tokenName = deposit.token?.symbol;
		} else {
			tokenName = '';
		}
	}
</script>

<div class="flex w-full max-w-2xl flex-col gap-6">
	<DeploymentSectionHeader
		title={`Deposit amount (${tokenName})`}
		description="The amount of tokens you want to deposit"
	/>
	{#if deposit.presets}
		<div class="flex w-full flex-wrap justify-center gap-4">
			{#each deposit.presets as preset}
				<ButtonSelectOption
					active={currentDeposit?.amount === preset}
					buttonText={preset}
					clickHandler={() => handlePresetClick(preset)}
				/>
			{/each}
		</div>
	{/if}

	<Input
		class="text-center text-lg"
		size="lg"
		placeholder="Enter deposit amount"
		bind:value={inputValue}
		on:input={(e) => handleInput(e)}
	/>
</div>
