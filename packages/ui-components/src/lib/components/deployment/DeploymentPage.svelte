<script lang="ts">
	import { DeploymentSteps } from './DeploymentSteps.svelte';

	export let dotrain: string;
	export let key: string;
	export let name: string;
	export let description: string;
</script>

<DeploymentSteps {dotrain} deployment={key} deploymentDetails={{ name, description }} />
