<script lang="ts">
	import { Input, Label } from 'flowbite-svelte';
	import type { OrderIO } from '@rainlanguage/orderbook/js_api';
	import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

	export let i: number;
	export let label: 'Input' | 'Output';
	export let vault: OrderIO;
	export let vaultIds: string[];
	export let gui: DotrainOrderGui;
</script>

<div class="flex w-full max-w-2xl flex-col gap-6">
	<div class="flex flex-col gap-4">
		<div class="flex flex-row gap-6">
			<Label class="whitespace-nowrap text-xl"
				>{label}
				{i + 1} ({vault.token?.symbol || 'Unknown'})</Label
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
