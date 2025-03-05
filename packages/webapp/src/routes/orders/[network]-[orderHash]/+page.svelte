<script lang="ts">
	import {
		OrderDetail,
		PageHeader,
		TransactionStatus,
		transactionStore
	} from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { codeMirrorTheme, lightweightChartsTheme, colorTheme } from '$lib/darkMode';
	import { handleDepositOrWithdrawModal, handleOrderRemoveModal } from '$lib/services/modal';
	import { useQueryClient } from '@tanstack/svelte-query';
	import { Toast } from 'flowbite-svelte';
	import { CheckCircleSolid } from 'flowbite-svelte-icons';
	import { fade } from 'svelte/transition';

	const queryClient = useQueryClient();
	const { orderHash, network } = $page.params;
	const { settings } = $page.data.stores;
	const orderbookAddress = $settings?.orderbooks[network]?.address;
	const subgraphUrl = $settings.subgraphs[network];
	const rpcUrl = $settings.networks[network]?.rpc;
	const chainId = $settings.networks[network]?.['chain-id'];

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
	{handleDepositOrWithdrawModal}
	{handleOrderRemoveModal}
/>
