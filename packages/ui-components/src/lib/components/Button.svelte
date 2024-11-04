<script lang="ts">
	const testAlert = () => {
		alert('test');
	};
</script>

<button class="border" on:click={testAlert}>Test</button>
