<script lang="ts">
	import {
		OrderDetail,
		PageHeader,
		TransactionStatus,
		transactionStore
	} from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { codeMirrorTheme, lightweightChartsTheme, colorTheme } from '$lib/darkMode';
	import { handleDepositOrWithdrawModal } from '$lib/services/modal';
	import { wagmiConfig } from '$lib/stores/wagmi';
	import { useQueryClient } from '@tanstack/svelte-query';
	import { Toast } from 'flowbite-svelte';
	import { CheckCircleSolid } from 'flowbite-svelte-icons';
	import { fade } from 'svelte/transition';
	import type { SgVault } from '@rainlanguage/orderbook/js_api';

	const queryClient = useQueryClient();
	const { orderHash, network } = $page.params;
	const { settings } = $page.data.stores;
	const orderbookAddress = $settings?.orderbooks[network]?.address;
	const subgraphUrl = $settings.subgraphs[network];
	const rpcUrl = $settings.networks[network]?.rpc;
	const chainId = $settings.networks[network]?.['chain-id'];

	let toastOpen: boolean = false;
	let toastMessage: string = 'Vault balance updated';
	let counter: number = 5;

	function triggerToast(message: string = 'Vault balance updated') {
		toastMessage = message;
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

	function handleVaultAction(
		event: CustomEvent<{ vault: SgVault }>,
		action: 'deposit' | 'withdraw'
	) {
		const { vault } = event.detail;

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
				subgraphUrl
			}
		});
	}

	const onDeposit = (event: CustomEvent<{ vault: SgVault }>) => handleVaultAction(event, 'deposit');
	const onWithdraw = (event: CustomEvent<{ vault: SgVault }>) =>
		handleVaultAction(event, 'withdraw');
</script>

<PageHeader title="Order" pathname={$page.url.pathname} />

{#if toastOpen}
	<Toast dismissable={true} position="top-right" transition={fade}>
		<CheckCircleSolid slot="icon" class="h-5 w-5" />
		{toastMessage}
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
	{wagmiConfig}
	on:deposit={onDeposit}
	on:withdraw={onWithdraw}
/>
