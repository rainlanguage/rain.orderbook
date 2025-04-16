<script lang="ts">
	import type { SgVault } from '@rainlanguage/orderbook';
	import { bigintStringToHex } from '../utils/hex';
	import Tooltip from './Tooltip.svelte';
	import { formatUnits } from 'viem';

	export let tokenVault: SgVault;
	export let subgraphName: string;
</script>

<div
	class="flex cursor-pointer items-center justify-between space-y-2 rounded-lg border border-gray-100 p-2"
	data-testid="vault-link"
>
	<div class="flex flex-col items-start gap-y-2">
		<Tooltip triggeredBy={`#token-info-${tokenVault.vaultId}`}>
			ID: <span class="font-mono">{bigintStringToHex(tokenVault.vaultId)}</span>
		</Tooltip>
		<a href={`/vaults/${subgraphName}-${tokenVault.id}`} id={`token-info-${tokenVault.vaultId}`}>
			{tokenVault.token.name} ({tokenVault.token.symbol})
		</a>
		<span class="text-sm text-gray-500 dark:text-gray-400">
			Balance: {formatUnits(
				BigInt(tokenVault.balance),
				parseInt(tokenVault.token.decimals || '18')
			)}
		</span>
	</div>
	<div>
		<slot name="buttons" />
	</div>
</div>
