<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { DeploymentPage } from '@rainlanguage/ui-components';

	const { dotrain, key, name, description } = $page.data;

	if (!dotrain || !key) {
		setTimeout(() => {
			goto('/deploy');
		}, 5000);
	}
</script>

{#if !dotrain || !key}
	<div>Deployment not found. Redirecting to deployments page...</div>
{:else}
	<DeploymentPage {dotrain} {key} {name} {description} />
{/if}
