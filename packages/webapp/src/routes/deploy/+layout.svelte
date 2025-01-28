<script lang="ts">
	import { PageHeader } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
</script>

<PageHeader title={$page.data.name || 'Deploy'} pathname={$page.url.pathname} />
<slot />
