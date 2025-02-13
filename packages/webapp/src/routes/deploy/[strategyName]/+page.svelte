<script lang="ts">
	import { page } from '$app/stores';
	import { PageHeader, StrategyPage } from '@rainlanguage/ui-components';

	const { dotrain, strategyName } = $page.data;
</script>

<PageHeader title={$page.data.strategyName || 'Deploy'} pathname={$page.url.pathname} />

<StrategyPage {dotrain} {strategyName} />
