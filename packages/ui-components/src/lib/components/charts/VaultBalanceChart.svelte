<script lang="ts">
	import { timestampSecondsToUTCTimestamp } from '../../utils/time';
	import { bigintToFloat } from '../../utils/number';
	import type { SgVault } from '@rainlanguage/orderbook/js_api';
	import { createQuery } from '@tanstack/svelte-query';
	import {
		getVaultBalanceChanges,
		type SgClearBounty,
		type SgDeposit,
		type SgTradeVaultBalanceChange,
		type SgWithdrawal
	} from '@rainlanguage/orderbook/js_api';
	import TanstackLightweightChartLine from '../charts/TanstackLightweightChartLine.svelte';
	import { QKEY_VAULT_CHANGES } from '../../queries/keys';

	export let vault: SgVault;
	export let id: string;
	export let subgraphUrl: string;
	export let lightweightChartsTheme;

	$: query = createQuery({
		queryKey: [id, QKEY_VAULT_CHANGES + id, QKEY_VAULT_CHANGES],
		queryFn: () => {
			return getVaultBalanceChanges(subgraphUrl || '', vault.id, {
				page: 1,
				pageSize: 1000
			});
		},
		enabled: !!subgraphUrl
	});

	const Chart = TanstackLightweightChartLine<
		SgWithdrawal | SgDeposit | SgTradeVaultBalanceChange | SgClearBounty
	>;

	$: console.log($query.data);
</script>

{#if vault && $query.data}
	<Chart
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
