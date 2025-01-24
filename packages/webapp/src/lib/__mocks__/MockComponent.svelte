<script lang="ts">
	import { props } from './MockComponent';

	// whenever the props update, we want to update the store with those
	$: $props = $$props;
</script>
