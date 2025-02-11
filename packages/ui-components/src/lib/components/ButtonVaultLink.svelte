<script lang="ts">
	import type { Vault } from '@rainlanguage/orderbook/js_api';
	import { bigintStringToHex } from '../utils/hex';
	import { Tooltip } from 'flowbite-svelte';
	import { formatUnits } from 'viem';

	export let tokenVault: Vault;
	export let subgraphName: string;
</script>

<a href={`/vaults/${subgraphName}-${tokenVault.id}`}>
	<div class="cursor-pointer rounded-lg" id="token-info" data-testid="vault-link">
		<div class="flex flex-col space-y-2">
			<div class="flex flex-col items-start justify-between lg:flex-row lg:items-center">
				<Tooltip triggeredBy="#token-info" class="w-96">
					ID: <span class="font-mono">{bigintStringToHex(tokenVault.vaultId)}</span>
				</Tooltip>
				<span class="font-medium">
					{tokenVault.token.name} ({tokenVault.token.symbol})
				</span>
				<div class="flex w-full justify-between md:w-auto">
					<div class="mb-2 flex justify-end text-sm text-gray-500 md:hidden dark:text-gray-400">
						<span>Balance</span>
					</div>
					<span class="text-sm text-gray-500 dark:text-gray-400">
						{formatUnits(BigInt(tokenVault.balance), parseInt(tokenVault.token.decimals || '18'))}
					</span>
				</div>
			</div>
		</div>
	</div>
</a>
