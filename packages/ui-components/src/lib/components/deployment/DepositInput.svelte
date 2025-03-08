<script lang="ts">
	import {
		DotrainOrderGui,
		type GuiDepositCfg,
		type TokenDeposit,
		type TokenInfo
	} from '@rainlanguage/orderbook/js_api';
	import { Input } from 'flowbite-svelte';
	import ButtonSelectOption from './ButtonSelectOption.svelte';
	import DeploymentSectionHeader from './DeploymentSectionHeader.svelte';
	import { CloseCircleSolid } from 'flowbite-svelte-icons';
	import { onMount } from 'svelte';
	import { useGui } from '$lib/hooks/useGui';
	export let deposit: GuiDepositCfg;
	const gui = useGui();

	let error: string = '';
	let currentDeposit: TokenDeposit | undefined;
	let inputValue: string = '';
	let tokenInfo: TokenInfo | null = null;

	$: console.log('deposit', deposit);

	onMount(() => {
		console.log('gui.getDeposits()', gui.getDeposits());
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
			gui.saveDeposit(deposit.token?.key, preset);
			currentDeposit = gui?.getDeposits().find((d) => d.token === deposit.token?.key);
		}
	}

	function handleInput(e: Event) {
		if (deposit.token?.key) {
			if (e.currentTarget instanceof HTMLInputElement) {
				inputValue = e.currentTarget.value;
				gui.saveDeposit(deposit.token.key, e.currentTarget.value);
				currentDeposit = gui?.getDeposits().find((d) => d.token === deposit.token?.key);
				console.log('currentDeposit', currentDeposit);
			}
		}
	}

	$: if (deposit.token?.key) {
		getTokenSymbol();
	}
</script>

<div class="flex w-full flex-col gap-6">
	<DeploymentSectionHeader
		title={tokenInfo?.symbol ? `Deposit amount (${tokenInfo?.symbol})` : 'Deposit amount'}
		description={tokenInfo?.symbol
			? `The amount of ${tokenInfo?.symbol} that you want to deposit.`
			: 'The amount that you want to deposit.'}
	/>

	<div class="flex w-full flex-col gap-6">
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
</div>
