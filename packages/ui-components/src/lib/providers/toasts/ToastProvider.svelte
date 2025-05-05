<script lang="ts">
	import { createToastsContext } from './context';

	// Initialize the store
	const toastsStore = createToastsContext();
</script>

<slot />
