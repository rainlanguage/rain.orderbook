<script lang="ts">
	import { PageHeader, TransactionStatus, transactionStore } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { VaultDetail } from '@rainlanguage/ui-components';

	import { handleDepositOrWithdrawModal } from '$lib/services/modal';
	import { Toast } from 'flowbite-svelte';
	import { CheckCircleSolid } from 'flowbite-svelte-icons';
	import { fade } from 'svelte/transition';
	import { useQueryClient } from '@tanstack/svelte-query';
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
	{handleDepositOrWithdrawModal}
/>
