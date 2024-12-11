<script lang="ts" generics="T">
	import { ButtonLoading, IconError } from '@rainlanguage/ui-components';
	let isLoading: boolean = true;
</script>

<div>
	<ButtonLoading color="primary" class="w-full px-2 py-1" size="lg" pill loading={isLoading}
		>Loading button</ButtonLoading
	>
	<IconError />
</div>
