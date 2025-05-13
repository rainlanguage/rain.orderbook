<script lang="ts">
	import { invalidateTanstackQueries, PageHeader, useAccount } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { VaultDetail } from '@rainlanguage/ui-components';
	import { handleDepositOrWithdrawModal } from '$lib/services/modal';
	import { Toast } from 'flowbite-svelte';
	import { CheckCircleSolid } from 'flowbite-svelte-icons';
	import { fade } from 'svelte/transition';
	import { useQueryClient } from '@tanstack/svelte-query';
	import type { SgVault } from '@rainlanguage/orderbook';
	import type { Hex } from 'viem';
	const queryClient = useQueryClient();
	import { lightweightChartsTheme } from '$lib/darkMode';

	const { settings, activeOrderbookRef, activeNetworkRef } = $page.data.stores;
	const { account } = useAccount();
	const network = $page.params.network;
	const subgraphUrl = $settings?.subgraphs?.[network]?.url || '';
	const chainId = $settings?.networks?.[network]?.chainId || 0;
	const rpcUrl = $settings?.networks?.[network]?.rpc || '';

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

	function handleVaultAction(vault: SgVault, action: 'deposit' | 'withdraw') {
		handleDepositOrWithdrawModal({
			open: true,
			args: {
				vault,
				onDepositOrWithdraw: () => {
					invalidateTanstackQueries(queryClient, [$page.params.id]);
					triggerToast();
				},
				action,
				chainId,
				rpcUrl,
				subgraphUrl,
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
	{activeNetworkRef}
	{activeOrderbookRef}
	{onDeposit}
	{onWithdraw}
/>
