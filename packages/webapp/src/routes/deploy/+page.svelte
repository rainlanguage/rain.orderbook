<script lang="ts">
	import { DeploymentSteps } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	const { files } = $page.data;
	$: console.log(files);
</script>

<div class="flex flex-col justify-center pb-12">
	<DeploymentSteps />
</div>
