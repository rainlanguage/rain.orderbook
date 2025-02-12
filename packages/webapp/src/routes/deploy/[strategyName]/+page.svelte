<script lang="ts">
	import { page } from '$app/stores';
	import { StrategySection } from '@rainlanguage/ui-components';

	const { dotrain, strategyName } = $page.data;
</script>

<StrategySection rawDotrain={dotrain} {strategyName} />
