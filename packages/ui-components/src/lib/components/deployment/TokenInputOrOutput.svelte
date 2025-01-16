<script lang="ts">
	import { Input, Label } from 'flowbite-svelte';
	import type { OrderIO, TokenInfo } from '@rainlanguage/orderbook/js_api';
	import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

	export let i: number;
	export let label: 'Input' | 'Output';
	export let vault: OrderIO;
	export let vaultIds: string[];
	export let gui: DotrainOrderGui;
	let tokenInfo: TokenInfo | null = null;

	const handleGetTokenInfo = async () => {
		if (!vault.token?.key) return;
		try {
			tokenInfo = await gui.getTokenInfo(vault.token?.key);
		} catch (e) {
			console.error('ERROR getting token info', e);
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
</div>
