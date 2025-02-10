<script lang="ts">
	import type { Vault } from '@rainlanguage/orderbook/js_api';
	import { formatUnits } from 'viem';
	import OrderOrVaultHash from './OrderOrVaultHash.svelte';

	export let tokenVault: Vault;
	export let subgraphName: string;
</script>

<div class="flex w-full flex-row justify-between">
	<div class="flex flex-col gap-2">
		<span class="font-medium">
			{tokenVault.token.name} ({tokenVault.token.symbol})
		</span>
		<OrderOrVaultHash orderOrVault={tokenVault} type="vaults" network={subgraphName} />
	</div>
	<div class="flex flex-col gap-2">
		<span>Balance</span>
		<span class="text-right text-sm text-gray-500 dark:text-gray-400">
			{formatUnits(BigInt(tokenVault.balance), parseInt(tokenVault.token.decimals || '18'))}
		</span>
	</div>
</div>
