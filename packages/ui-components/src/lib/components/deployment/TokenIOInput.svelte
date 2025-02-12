<script lang="ts">
	import { AccordionItem, Input } from 'flowbite-svelte';
	import type { OrderIO, TokenInfo, DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
	import { CloseCircleSolid } from 'flowbite-svelte-icons';
	import DeploymentSectionHeader from './DeploymentSectionHeader.svelte';

	export let isInput: boolean;
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
			tokenInfo = await gui?.getTokenInfo(vault.token?.key);
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

	export let open = true;
</script>

<AccordionItem {open}>
	<span slot="header">
		<DeploymentSectionHeader
			title={`${label} ${i + 1} ${tokenInfo?.symbol ? `(${tokenInfo.symbol})` : ''}`}
			description={`${tokenInfo?.symbol} Vault ID`}
			{open}
			value={undefined}
		/>
	</span>

	<div class="flex w-full max-w-2xl flex-col gap-6">
		<Input
			size="lg"
			type="text"
			placeholder="Enter vault ID"
			bind:value={vaultIds[i]}
			on:change={() => gui?.setVaultId(isInput, i, vaultIds[i])}
		/>

		{#if error}
			<div class="flex h-5 flex-row items-center gap-2">
				<CloseCircleSolid class="h-5 w-5" color="red" />
				<span>{error}</span>
			</div>
		{/if}
	</div></AccordionItem
>
