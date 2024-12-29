<script lang="ts">
	import { OrderDetail, PageHeader } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';

	import { codeMirrorTheme, lightweightChartsTheme, colorTheme } from '$lib/darkMode';
	const { id, network } = $page.params;
	const { settings } = $page.data.stores;
	const orderbookAddress = $settings?.orderbooks[network]?.address;
	const subgraphUrl = $settings.subgraphs[network];
	const rpcUrl = $settings.networks[network]?.rpc;
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
/>
