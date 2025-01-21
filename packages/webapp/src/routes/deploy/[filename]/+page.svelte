<script lang="ts">
	import { DeploymentSteps } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { CheckCircleOutline } from 'flowbite-svelte-icons';
	import { fade } from 'svelte/transition';

	const { dotrain, strategyName, errorDetails, error } = $page.data;
	let _dotrain = dotrain;
</script>

{#if dotrain}
	<div in:fade class="flex flex-col gap-4">
		<div class="mb-6 flex items-center gap-2">
			<CheckCircleOutline class="h-5 w-5 text-green-500" />
			<p>Strategy loaded: {strategyName}</p>
		</div>

		<DeploymentSteps dotrain={_dotrain} />
	</div>
{:else}
	<div>
		<p>{error}</p>
		<p>{errorDetails}</p>
	</div>
{/if}
