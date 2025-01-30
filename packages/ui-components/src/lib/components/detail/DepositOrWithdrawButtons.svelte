<script lang="ts">
	import type { Vault } from '@rainlanguage/orderbook/js_api';
	import type { CreateQueryResult } from '@tanstack/svelte-query';
	import { Button } from 'flowbite-svelte';
	import { ArrowDownOutline, ArrowUpOutline } from 'flowbite-svelte-icons';

	export let handleDepositOrWithdrawModal: (args: {
		vault: Vault;
		onDepositOrWithdraw: () => void;
		action: 'deposit' | 'withdraw';
		subgraphUrl: string;
		chainId: number;
		rpcUrl: string;
	}) => void;

	export let vault: Vault;
	export let subgraphUrl: string;
	export let chainId: number;
	export let rpcUrl: string;
	export let query: CreateQueryResult;
</script>

<Button
	data-testid="vaultDetailDepositButton"
	color="dark"
	on:click={() =>
		handleDepositOrWithdrawModal({
			vault,
			onDepositOrWithdraw: $query.refetch,
			action: 'deposit',
			subgraphUrl,
			chainId,
			rpcUrl
		})}><ArrowDownOutline size="xs" class="mr-2" />Deposit</Button
>
<Button
	data-testid="vaultDetailDepositButton"
	color="dark"
	on:click={() =>
		handleDepositOrWithdrawModal({
			vault,
			onDepositOrWithdraw: $query.refetch,
			action: 'withdraw',
			subgraphUrl,
			chainId,
			rpcUrl
		})}><ArrowUpOutline size="xs" class="mr-2" />Withdraw</Button
>
