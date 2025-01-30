<script lang="ts">
	import { PageHeader } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { VaultDetail } from '@rainlanguage/ui-components';
	import { wagmiConfig } from '$lib/stores/wagmi';
	import { handleDepositOrWithdrawModal } from '$lib/services/modal';
	const { settings, activeOrderbookRef, activeNetworkRef, lightweightChartsTheme } =
		$page.data.stores;
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
/>
