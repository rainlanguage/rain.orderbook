<script lang="ts">
	import { PageHeader } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';

	$: console.log($page.data.name);
</script>

<PageHeader title={$page.data.name || 'Deploy'} pathname={$page.url.pathname} />
<slot />
