<script lang="ts">
	import { page } from '$app/stores';
	import { StrategyPage } from '@rainlanguage/ui-components';
	const { dotrain, strategyName } = $page.data;
</script>

<StrategyPage {dotrain} {strategyName} />
