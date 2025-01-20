<script lang="ts">
	import {
		DotrainOrderGui,
		type GuiDeposit,
		type TokenDeposit,
		type TokenInfo
	} from '@rainlanguage/orderbook/js_api';
	import { Input } from 'flowbite-svelte';
	import ButtonSelectOption from './ButtonSelectOption.svelte';
	import DeploymentSectionHeader from './DeploymentSectionHeader.svelte';
	import { CloseCircleSolid } from 'flowbite-svelte-icons';

	let error: string = '';
	export let deposit: GuiDeposit;
	export let gui: DotrainOrderGui;

	let currentDeposit: TokenDeposit | undefined;
	let inputValue: string = '';
	let tokenInfo: TokenInfo | null = null;

	const getTokenSymbol = async () => {
		if (!deposit.token?.key) return;
		try {
			tokenInfo = await gui.getTokenInfo(deposit.token?.key);
		} catch {
			error = 'Error getting token details';
		}
	};

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

	$: if (deposit.token?.key) {
		getTokenSymbol();
	}
</script>

<div class="flex w-full max-w-2xl flex-col gap-6">
	<DeploymentSectionHeader
		title={tokenInfo?.symbol ? `Deposit amount (${tokenInfo?.symbol})` : 'Deposit amount'}
		description={tokenInfo?.symbol
			? `The amount of ${tokenInfo?.symbol} that you want to deposit`
			: 'The amount that you want to deposit'}
	/>
	{#if deposit.presets}
		<div class="flex w-full flex-wrap gap-4">
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
		size="lg"
		placeholder="Enter deposit amount"
		bind:value={inputValue}
		on:input={(e) => handleInput(e)}
	/>

	{#if error}
		<div class="flex h-5 flex-row items-center gap-2">
			<CloseCircleSolid class="h-5 w-5" color="red" />
			<span>{error}</span>
		</div>
	{/if}
</div>
