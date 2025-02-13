<script lang="ts">
	import { PageHeader, TransactionStatus, transactionStore } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { VaultDetail } from '@rainlanguage/ui-components';
	import { wagmiConfig, signerAddress } from '$lib/stores/wagmi';
	import { handleDepositOrWithdrawModal } from '$lib/services/modal';
	import { useQueryClient } from '@tanstack/svelte-query';
	const queryClient = useQueryClient();

	const { settings, activeOrderbookRef, activeNetworkRef, lightweightChartsTheme } =
		$page.data.stores;

	$: if ($transactionStore.status === TransactionStatus.SUCCESS) {
		console.log('invalidating vault detail', $page.params.id);
		queryClient.invalidateQueries({
			queryKey: [$page.params.id]
		});
	}
</script>

<PageHeader title="Vault" pathname={$page.url.pathname} />

<VaultDetail
	id={$page.params.id}
	network={$page.params.network}
	{lightweightChartsTheme}
	{settings}
	{activeNetworkRef}
	{activeOrderbookRef}
	{wagmiConfig}
	{handleDepositOrWithdrawModal}
	{signerAddress}
/>
