<script lang="ts">
	import { Input } from 'flowbite-svelte';
	import {
		type OrderIO,
		type TokenInfo,
		type DotrainOrderGui
	} from '@rainlanguage/orderbook/js_api';
	import DeploymentSectionHeader from './DeploymentSectionHeader.svelte';
	import { onMount } from 'svelte';

	export let i: number;
	export let label: 'Input' | 'Output';
	export let vault: OrderIO;
	export let gui: DotrainOrderGui;
	export let handleUpdateGuiState: (gui: DotrainOrderGui) => void;

	let tokenInfo: TokenInfo | null = null;
	let inputValue: string = '';
	let error: string = '';

	onMount(() => {
		if (!gui) return;
		const vaultIds = gui.getVaultIds();
		if (label === 'Input') {
			inputValue = vaultIds.get('input')?.[i];
		} else if (label === 'Output') {
			inputValue = vaultIds.get('output')?.[i];
		}
	});

	const handleGetTokenInfo = async () => {
		if (!vault.token?.key) return;
		try {
			tokenInfo = await gui?.getTokenInfo(vault.token?.key);
		} catch (e) {
			const errorMessage = (e as Error).message
				? (e as Error).message
				: 'Error getting token info.';
			error = errorMessage;
		}
	};

	const handleInput = async () => {
		const isInput = label === 'Input';
		try {
			gui?.setVaultId(isInput, i, inputValue);
			handleUpdateGuiState(gui);
		} catch (e) {
			const errorMessage = (e as Error).message ? (e as Error).message : 'Error setting vault ID.';
			error = errorMessage;
		}
	};

	$: if (vault.token?.key) {
		handleGetTokenInfo();
	}
</script>

<div class="flex w-full flex-col gap-6">
	<DeploymentSectionHeader
		title={`${label} ${i + 1} ${tokenInfo?.symbol ? `(${tokenInfo.symbol})` : ''}`}
		description={`${tokenInfo?.symbol} vault ID`}
	/>
	<div class="flex flex-col gap-2">
		<Input
			size="lg"
			type="text"
			placeholder="Enter vault ID"
			bind:value={inputValue}
			on:input={handleInput}
		/>
		{#if error}
			<p class="text-red-500">{error}</p>
		{/if}
	</div>
</div>
