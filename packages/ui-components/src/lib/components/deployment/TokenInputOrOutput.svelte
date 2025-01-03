<script lang="ts">
	import { Label, Input } from 'flowbite-svelte';
	import type { TokenInfos, Vault } from '@rainlanguage/orderbook/js_api';
	import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

	export let i: number;
	export let label: 'Input' | 'Output';
	export let vault: Vault;
	export let tokenInfos: TokenInfos;
	export let vaultIds: string[];
	export let gui: DotrainOrderGui;
</script>

<div class="flex flex-col items-center gap-2">
	<Label class="whitespace-nowrap"
		>{label} {i + 1} ({tokenInfos.get(vault.token.address)?.symbol || ''})</Label
	>
	<Input
		type="text"
		placeholder="Enter vault ID"
		bind:value={vaultIds[i]}
		on:change={() => gui?.setVaultId(true, i, vaultIds[i])}
	/>
</div>
