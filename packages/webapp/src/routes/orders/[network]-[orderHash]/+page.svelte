<script lang="ts">
	import {
		OrderDetail,
		PageHeader,
		TransactionStatus,
		transactionStore,
		useAccount
	} from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { codeMirrorTheme, lightweightChartsTheme, colorTheme } from '$lib/darkMode';
	import { handleDepositOrWithdrawModal, handleOrderRemoveModal } from '$lib/services/modal';
	import { useQueryClient } from '@tanstack/svelte-query';
	import { Toast } from 'flowbite-svelte';
	import { CheckCircleSolid } from 'flowbite-svelte-icons';
	import { fade } from 'svelte/transition';
	import type { SgVault } from '@rainlanguage/orderbook/js_api';
	import type { Hex } from 'viem';

	const queryClient = useQueryClient();
	const { orderHash, network } = $page.params;
	const { settings } = $page.data.stores;
	const orderbookAddress = $settings?.orderbooks[network]?.address;
	const subgraphUrl = $settings.subgraphs[network];
	const rpcUrl = $settings.networks[network]?.rpc;
	const chainId = $settings.networks[network]?.['chain-id'];
	const { account } = useAccount();

	let toastOpen: boolean = false;
	let counter: number = 5;

	function triggerToast() {
		toastOpen = true;
		counter = 5;
		timeout();
	}

	function timeout() {
		if (--counter > 0) return setTimeout(timeout, 1000);
		toastOpen = false;
	}

	$: if ($transactionStore.status === TransactionStatus.SUCCESS) {
		queryClient.invalidateQueries({
			queryKey: [orderHash],
			refetchType: 'all',
			exact: false
		});
		triggerToast();
	}

	function handleVaultAction(vault: SgVault, action: 'deposit' | 'withdraw') {
		const network = $page.params.network;
		const orderHash = $page.params.orderHash;
		const subgraphUrl = $settings?.subgraphs?.[network] || '';
		const chainId = $settings?.networks?.[network]?.['chain-id'] || 0;

		handleDepositOrWithdrawModal({
			open: true,
			args: {
				vault,
				onDepositOrWithdraw: () => {
					queryClient.invalidateQueries({
						queryKey: [orderHash],
						refetchType: 'all',
						exact: false
					});
				},
				action,
				chainId,
				rpcUrl,
				subgraphUrl,
				// Casting to Hex since the buttons cannot appear if account is null
				account: $account as Hex
			}
		});
	}

	function onDeposit(vault: SgVault) {
		handleVaultAction(vault, 'deposit');
	}

	function onWithdraw(vault: SgVault) {
		handleVaultAction(vault, 'withdraw');
	}
</script>

<PageHeader title="Order" pathname={$page.url.pathname} />

{#if toastOpen}
	<Toast dismissable={true} position="top-right" transition={fade}>
		<CheckCircleSolid slot="icon" class="h-5 w-5" />
		Vault balance updated
		<span class="text-sm text-gray-500">Autohide in {counter}s.</span>
	</Toast>
{/if}

<OrderDetail
	{orderHash}
	{subgraphUrl}
	{rpcUrl}
	{codeMirrorTheme}
	{lightweightChartsTheme}
	{colorTheme}
	{orderbookAddress}
	{chainId}
	{onDeposit}
	{onWithdraw}
	{handleOrderRemoveModal}
/>
