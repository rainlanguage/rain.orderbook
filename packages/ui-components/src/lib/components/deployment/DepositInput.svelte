<script lang="ts">
	import {
		DotrainOrderGui,
		type GuiDeposit,
		type TokenDeposit,
		type TokenInfo
	} from '@rainlanguage/orderbook/js_api';
	import { AccordionItem, Input } from 'flowbite-svelte';
	import ButtonSelectOption from './ButtonSelectOption.svelte';
	import DeploymentSectionHeader from './DeploymentSectionHeader.svelte';
	import { CloseCircleSolid } from 'flowbite-svelte-icons';
	import { onMount } from 'svelte';

	export let deposit: GuiDeposit;
	export let gui: DotrainOrderGui;
	export let open: boolean = true;

	let error: string = '';
	let currentDeposit: TokenDeposit | undefined;
	let inputValue: string = '';
	let tokenInfo: TokenInfo | null = null;

	onMount(() => {
		setCurrentDeposit();
	});

	const setCurrentDeposit = async () => {
		try {
			currentDeposit = gui.getDeposits().find((d) => d.token === deposit.token?.key);
			inputValue = currentDeposit?.amount || '';
		} catch {
			currentDeposit = undefined;
		}
	};

	const getTokenSymbol = async () => {
		if (!deposit.token?.key) return;
		try {
			tokenInfo = await gui?.getTokenInfo(deposit.token?.key);
		} catch (e) {
			const errorMessage = (e as Error).message
				? (e as Error).message
				: 'Error getting token info.';
			error = errorMessage;
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

<AccordionItem bind:open>
	<span slot="header" class="w-full">
		<DeploymentSectionHeader
			title={tokenInfo?.symbol ? `Deposit amount (${tokenInfo?.symbol})` : 'Deposit amount'}
			description={tokenInfo?.symbol
				? `The amount of ${tokenInfo?.symbol} that you want to deposit.`
				: 'The amount that you want to deposit.'}
			{open}
			value={currentDeposit?.amount}
		/>
	</span>

	<div class="flex w-full max-w-2xl flex-col gap-6">
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
</AccordionItem>
