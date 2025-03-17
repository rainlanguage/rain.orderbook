<script lang="ts">
	import type { SgVault } from '@rainlanguage/orderbook/js_api';
	import { Button } from 'flowbite-svelte';
	import { ArrowDownOutline, ArrowUpOutline } from 'flowbite-svelte-icons';
	import type { DepositOrWithdrawModalProps } from '../../types/modal';
	import type { CreateQueryResult } from '@tanstack/svelte-query';

	export let vault: SgVault;
	export let chainId: number;
	export let rpcUrl: string;
	export let subgraphUrl: string;
	export let query: CreateQueryResult;
	export let handleDepositOrWithdrawModal: (props: DepositOrWithdrawModalProps) => void;
	export let onDepositOrWithdraw: () => void = $query.refetch;
</script>

<div class="flex gap-x-2">
	<Button
		data-testid="depositOrWithdrawButton"
		color="light"
		size="xs"
		on:click={() =>
			handleDepositOrWithdrawModal({
				open: true,
				args: {
					vault,
					onDepositOrWithdraw,
					action: 'deposit',
					chainId,
					rpcUrl,
					subgraphUrl
				}
			})}><ArrowUpOutline size="xs" /></Button
	>
	<Button
		data-testid="depositOrWithdrawButton"
		color="light"
		size="xs"
		on:click={() =>
			handleDepositOrWithdrawModal({
				open: true,
				args: {
					vault,
					onDepositOrWithdraw,
					action: 'withdraw',
					chainId,
					rpcUrl,
					subgraphUrl
				}
			})}><ArrowDownOutline size="xs" /></Button
	>
</div>
