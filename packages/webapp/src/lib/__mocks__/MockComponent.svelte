<script lang="ts">
	import { props } from './MockComponent';

	// Add test ID to the component props
	export let testId: string = '';

	// whenever the props update, we want to update the store with those
	$: $props = $$props;
</script>

<div data-testid={testId}></div>
