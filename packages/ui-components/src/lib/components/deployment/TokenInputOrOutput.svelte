<script lang="ts">
	import { Input, Label } from 'flowbite-svelte';
	import type { OrderIO, TokenInfo, DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
	import { CloseCircleSolid } from 'flowbite-svelte-icons';

	export let i: number;
	export let label: 'Input' | 'Output';
	export let vault: OrderIO;
	export let vaultIds: string[];
	export let gui: DotrainOrderGui;
	let error: string = '';
	let tokenInfo: TokenInfo | null = null;

	const handleGetTokenInfo = async () => {
		if (!vault.token?.key) return;
		try {
			tokenInfo = await gui.getTokenInfo(vault.token?.key);
		} catch (e) {
			const errorMessage = (e as Error).message
				? (e as Error).message
				: 'Error getting token info.';
			error = errorMessage;
		}
	};

	$: if (vault.token?.key) {
		handleGetTokenInfo();
	}
</script>

<div class="flex w-full max-w-2xl flex-col gap-6">
	<div class="flex flex-col gap-4">
		<div class="flex flex-row gap-6">
			<Label class="whitespace-nowrap text-xl"
				>{label}
				{i + 1}
				{tokenInfo?.symbol ? `(${tokenInfo.symbol})` : ''}</Label
			>
		</div>
		<Input
			size="lg"
			type="text"
			placeholder="Enter vault ID"
			bind:value={vaultIds[i]}
			on:change={() => gui?.setVaultId(true, i, vaultIds[i])}
		/>
	</div>
	{#if error}
		<div class="flex h-5 flex-row items-center gap-2">
			<CloseCircleSolid class="h-5 w-5" color="red" />
			<span>{error}</span>
		</div>
	{/if}
</div>
