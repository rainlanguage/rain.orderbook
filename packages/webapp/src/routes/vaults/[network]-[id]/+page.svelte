<script lang="ts">
	import { PageHeader, TransactionStatus, transactionStore } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { VaultDetail } from '@rainlanguage/ui-components';
	import { signerAddress } from '$lib/stores/wagmi';
	import { handleDepositOrWithdrawModal } from '$lib/services/modal';
	import { Toast } from 'flowbite-svelte';
	import { CheckCircleSolid } from 'flowbite-svelte-icons';
	import { fade } from 'svelte/transition';
	import { useQueryClient } from '@tanstack/svelte-query';
	import type { SgVault } from '@rainlanguage/orderbook/js_api';

	const queryClient = useQueryClient();

	const { settings, activeOrderbookRef, activeNetworkRef, lightweightChartsTheme } =
		$page.data.stores;

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

	function onDeposit(event: CustomEvent<{ vault: SgVault }>) {
		const { vault } = event.detail;

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
						queryKey: [$page.params.id],
						refetchType: 'all',
						exact: false
					});
				},
				action: 'deposit',
				chainId,
				rpcUrl,
				subgraphUrl
			}
		});
	}

	function onWithdraw(event: CustomEvent<{ vault: SgVault }>) {
		const { vault } = event.detail;

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
						queryKey: [$page.params.id],
						refetchType: 'all',
						exact: false
					});
				},
				action: 'withdraw',
				chainId,
				rpcUrl,
				subgraphUrl
			}
		});
	}

	$: if ($transactionStore.status === TransactionStatus.SUCCESS) {
		queryClient.invalidateQueries({
			queryKey: [$page.params.id],
			refetchType: 'all',
			exact: false
		});
		triggerToast();
	}
</script>

<PageHeader title="Vault" pathname={$page.url.pathname} />

{#if toastOpen}
	<Toast dismissable={true} position="top-right" transition={fade}>
		<CheckCircleSolid slot="icon" class="h-5 w-5" />
		Vault balance updated
		<span class="text-sm text-gray-500">Autohide in {counter}s.</span>
	</Toast>
{/if}

<VaultDetail
	id={$page.params.id}
	network={$page.params.network}
	{lightweightChartsTheme}
	{settings}
	{signerAddress}
	{activeNetworkRef}
	{activeOrderbookRef}
	on:deposit={onDeposit}
	on:withdraw={onWithdraw}
/>
