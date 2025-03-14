<script lang="ts">
	import {
		PageHeader,
		TransactionStatus,
		transactionStore,
		DepositOrWithdrawButtons,
		VaultDetail
	} from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { wagmiConfig, signerAddress } from '$lib/stores/wagmi';
	import { handleDepositOrWithdrawModal } from '$lib/services/modal';
	import { CheckCircleSolid } from 'flowbite-svelte-icons';
	import { useQueryClient } from '@tanstack/svelte-query';
	import { Toast } from 'flowbite-svelte';
	import { fade } from 'svelte/transition';
	import { derived } from 'svelte/store';

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

	$: if ($transactionStore.status === TransactionStatus.SUCCESS) {
		queryClient.invalidateQueries({
			queryKey: [$page.params.id],
			refetchType: 'all',
			exact: false
		});
		triggerToast();
	}

	const isCurrentUserOwner = derived(signerAddress, ($signerAddress) => {
		return (owner: string) => $signerAddress === owner;
	});
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
	{isCurrentUserOwner}
>
	<svelte:fragment slot="action-buttons" let:data let:query>
		<DepositOrWithdrawButtons vault={data} />
	</svelte:fragment>
</VaultDetail>
