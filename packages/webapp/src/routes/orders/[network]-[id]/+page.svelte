<script lang="ts">
	import { OrderDetail, PageHeader } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';

	const { colorTheme, codeMirrorTheme, lightweightChartsTheme } = $page.data.stores;
	const { id, network } = $page.params;
	const { settings } = $page.data.stores;
	const subgraphUrl = $settings.subgraphs[network];
	const rpcUrl = $settings.networks[network]?.rpc;
</script>

<PageHeader title="Order" pathname={$page.url.pathname} />
<OrderDetail {id} {subgraphUrl} {rpcUrl} {colorTheme} {codeMirrorTheme} {lightweightChartsTheme} />
