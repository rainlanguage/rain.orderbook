<script lang="ts">
	import { Input } from 'flowbite-svelte';
	import { type OrderIOCfg, type TokenInfo } from '@rainlanguage/orderbook';
	import DeploymentSectionHeader from './DeploymentSectionHeader.svelte';
	import { onMount } from 'svelte';
	import { useGui } from '$lib/hooks/useGui';

	const gui = useGui();

	export let i: number;
	export let label: 'Input' | 'Output';
	export let vault: OrderIOCfg;

	let tokenInfo: TokenInfo | null = null;
	let inputValue: string = '';
	let error: string = '';

	onMount(() => {
		const result = gui.getVaultIds();
		if (result.error) {
			error = result.error.msg;
			return;
		}
		const vaultIds = result.value;
		if (label === 'Input') {
			inputValue = vaultIds.get('input')?.[i] as unknown as string;
		} else if (label === 'Output') {
			inputValue = vaultIds.get('output')?.[i] as unknown as string;
		}
	});

	const handleGetTokenInfo = async () => {
		if (!vault.token?.key) return;
		try {
			let result = await gui.getTokenInfo(vault.token?.key);
			if (result.error) {
				error = result.error.msg;
				return;
			}
			tokenInfo = result.value;
		} catch (e) {
			const errorMessage = (e as Error).message
				? (e as Error).message
				: 'Error getting token info.';
			error = errorMessage;
		}
	};

	const handleInput = async () => {
		const isInput = label === 'Input';
		error = '';
		try {
			gui?.setVaultId(isInput, i, inputValue);
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
