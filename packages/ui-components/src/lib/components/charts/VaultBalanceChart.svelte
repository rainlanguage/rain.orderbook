<script lang="ts">
	import { timestampSecondsToUTCTimestamp } from '../../utils/time';
	import { bigintToFloat } from 'tauri-app/src/lib/utils/number.ts';
	import { createQuery } from '@tanstack/svelte-query';
	import { vaultBalanceChangesList } from 'tauri-app/src/lib/queries/vaultBalanceChangesList.ts';
	import TanstackLightweightChartLine from './TanstackLightweightChartLine.svelte';
	import { QKEY_VAULT_CHANGES } from '../../queries/keys';
	import type {Readable} from "svelte/store";
	export let vault;
	export let subgraphUrl: string;
	export let lightweightChartsTheme: Readable<Record<string, unknown>>;
	$: query = createQuery({
		queryKey: [QKEY_VAULT_CHANGES, vault.id],
		queryFn: () => {
			return vaultBalanceChangesList(vault.id, subgraphUrl || '', 0, 1000);
		},
		enabled: !!subgraphUrl
	});
</script>

{#if vault}
	<TanstackLightweightChartLine
		title="Balance history"
		priceSymbol={vault.token.symbol}
		{query}
		timeTransform={(d) => timestampSecondsToUTCTimestamp(BigInt(d.timestamp))}
		valueTransform={(d) =>
			bigintToFloat(BigInt(d.newVaultBalance), Number(vault.token.decimals ?? 0))}
		emptyMessage="No deposits or withdrawals found"
		{lightweightChartsTheme}
	/>
{/if}
