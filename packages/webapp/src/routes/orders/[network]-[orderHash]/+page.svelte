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
	import { wagmiConfig } from '$lib/stores/wagmi';
	import { useQueryClient } from '@tanstack/svelte-query';
	import { Toast } from 'flowbite-svelte';
	import { CheckCircleSolid } from 'flowbite-svelte-icons';
	import { fade } from 'svelte/transition';
	import type { SgVault } from '@rainlanguage/orderbook/js_api';
	import { onDestroy } from 'svelte';

	const queryClient = useQueryClient();
	const { orderHash, network } = $page.params;
	const { settings } = $page.data.stores;
	const orderbookAddress = $settings?.orderbooks[network]?.address;
	const subgraphUrl = $settings.subgraphs[network];
	const rpcUrl = $settings.networks[network]?.rpc;
	const chainId = $settings.networks[network]?.['chain-id'];
	const { account } = useAccount();

	let toastOpen: boolean = false;
	let toastMessage: string = 'Vault balance updated';
	let counter: number = 5;
	let toastTimer: ReturnType<typeof setTimeout> | null = null;

	function triggerToast(message: string = 'Vault balance updated') {
		if (toastTimer) {
			clearTimeout(toastTimer);
			toastTimer = null;
		}

		toastMessage = message;
		toastOpen = true;
		counter = 5;
		startToastTimer();
	}

	function startToastTimer() {
		if (toastTimer) {
			clearTimeout(toastTimer);
		}

		toastTimer = setTimeout(() => {
			if (counter > 0) {
				counter--;
				startToastTimer();
			} else {
				toastOpen = false;
				toastTimer = null;
			}
		}, 1000);
	}

	onDestroy(() => {
		if (toastTimer) {
			clearTimeout(toastTimer);
			toastTimer = null;
		}
	});

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
		const subgraphUrl = $settings?.subgraphs?.[network] || '';
		const chainId = $settings?.networks?.[network]?.['chain-id'] || 0;
		const rpcUrl = $settings?.networks?.[network]?.['rpc'] || '';

		handleDepositOrWithdrawModal({
			open: true,
			args: {
				vault,
				onDepositOrWithdraw: () => {
					queryClient.invalidateQueries({
						queryKey: [$page.params.orderHash],
						refetchType: 'all',
						exact: false
					});
				},
				action,
				chainId,
				rpcUrl,
				subgraphUrl,
				account: $account
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
	{onDeposit}
	{onWithdraw}
	{handleOrderRemoveModal}
/>
