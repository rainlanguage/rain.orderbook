<script lang="ts">
	import { page } from '$app/stores';
	import { DeploymentSteps } from '@rainlanguage/ui-components';
</script>


<div class="flex h-screen flex-col gap-4">
	<DeploymentSteps filename={$page.data.filename} stateFromUrl={$page.data.state} />
</div>
