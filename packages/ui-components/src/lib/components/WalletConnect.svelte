<script lang="ts">
	import { page } from '$app/stores';
	import { Button } from 'flowbite-svelte';

	const { appKitModal } = $page.data;

	function handleClick() {
		$appKitModal.open();
	}
</script>

<Button on:click={handleClick}>Open Modal</Button>
<appkit-button />
<appkit-network-button />
