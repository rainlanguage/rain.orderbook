<script lang="ts">
	import { OrderDetail, PageHeader } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';

	import { codeMirrorTheme, lightweightChartsTheme, colorTheme } from '$lib/darkMode';
	import { handleDepositOrWithdrawModal } from '$lib/services/modal';
	import { wagmiConfig } from '$lib/stores/wagmi';
	const { id, network } = $page.params;
	const { settings } = $page.data.stores;
	const orderbookAddress = $settings?.orderbooks[network]?.address;
	const subgraphUrl = $settings.subgraphs[network];
	const rpcUrl = $settings.networks[network]?.rpc;
	const chainId = $settings.networks[network]?.['chain-id'];
</script>

<PageHeader title="Order" pathname={$page.url.pathname} />
<OrderDetail
	{id}
	{subgraphUrl}
	{rpcUrl}
	{codeMirrorTheme}
	{lightweightChartsTheme}
	{colorTheme}
	{orderbookAddress}
	{chainId}
	{wagmiConfig}
	{handleDepositOrWithdrawModal}
/>
