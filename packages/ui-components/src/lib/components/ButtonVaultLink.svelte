<script lang="ts">
	import type { Address, RaindexVault } from '@rainlanguage/orderbook';
	import { bigintToHex } from '../utils/hex';
	import Tooltip from './Tooltip.svelte';
	import { formatUnits } from 'viem';

	export let tokenVault: RaindexVault;
	export let chainId: number;
	export let orderbookAddress: Address;
</script>

<div
	class="flex cursor-pointer items-center justify-between space-y-2 rounded-lg border border-gray-100 p-2"
	data-testid="vault-link"
>
	<div class="flex flex-col items-start gap-y-2">
		<Tooltip triggeredBy={`#token-info-${tokenVault.vaultId}`}>
			ID: <span class="font-mono">{bigintToHex(tokenVault.vaultId)}</span>
		</Tooltip>
		<a
			href={`/vaults/${chainId}-${orderbookAddress}-${tokenVault.id}`}
			id={`token-info-${tokenVault.vaultId}`}
		>
			{tokenVault.token.name} ({tokenVault.token.symbol})
		</a>
		<span class="text-sm text-gray-500 dark:text-gray-400">
			Balance: {formatUnits(tokenVault.balance, Number(tokenVault.token.decimals) || 18)}
		</span>
	</div>
	<div>
		<slot name="buttons" />
	</div>
</div>
