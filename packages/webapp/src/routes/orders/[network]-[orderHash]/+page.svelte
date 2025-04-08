<script lang="ts">
	import { OrderDetail, PageHeader, useAccount } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { codeMirrorTheme, lightweightChartsTheme, colorTheme } from '$lib/darkMode';
	import { handleDepositOrWithdrawModal, handleOrderRemoveModal } from '$lib/services/modal';
	import { useQueryClient } from '@tanstack/svelte-query';
	import { Toast } from 'flowbite-svelte';
	import { CheckCircleSolid } from 'flowbite-svelte-icons';
	import { fade } from 'svelte/transition';
	import type { SgOrder } from '@rainlanguage/orderbook/js_api';

	const queryClient = useQueryClient();
	const { orderHash, network } = $page.params;
	const { settings } = $page.data.stores;
	const orderbookAddress = $settings?.orderbooks[network]?.address;
	const subgraphUrl = $settings.subgraphs[network];
	const rpcUrl = $settings.networks[network]?.rpc;
	const chainId = $settings.networks[network]?.['chain-id'];
	const { account } = useAccount();

	import { prepareOrderRemoval } from '$lib/services/handleRemoveOrder'; // Adjust the import path as needed
	import type { Hex } from 'viem';

	let toastOpen: boolean = false;
	let counter: number = 5;
	let toastMessage: string = '';

	function triggerToast(message: string) {
		toastMessage = message;
		toastOpen = true;
		counter = 5;
		timeout();
	}

	function timeout() {
		if (--counter > 0) return setTimeout(timeout, 1000);
		toastOpen = false;
	}

	function onRemove(event: CustomEvent<{ order: SgOrder }>) {
		const { order } = event.detail;

		handleOrderRemoveModal({
			open: true,
			args: {
				order,
				chainId,
				orderbookAddress,
				subgraphUrl,
				onRemove: () => {
					queryClient.invalidateQueries({
						queryKey: [orderHash],
						refetchType: 'all',
						exact: false
					});
					triggerToast('Order removed successfully');
				}
			}
		});
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
	{handleDepositOrWithdrawModal}
	{onRemove}
/>
