<script lang="ts">
	import icon from '$lib/assets/ledger.svg?raw';
</script>

<!-- eslint-disable-next-line svelte/no-at-html-tags -->
<div class="icon-ledger">{@html icon}</div>

<style lang="css">
	:global(.icon-ledger svg) {
		height: 1.5rem;
		width: auto;
		fill: white;
	}
</style>
