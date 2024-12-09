<script>
	import CodeMirrorRainlang from '../lib/components/CodeMirrorRainlang.svelte';
	export let props;
</script>

<CodeMirrorRainlang {...props}>
	<svelte:fragment slot="codemirror" let:value>
		<div data-testid="test-value">{value}</div>
	</svelte:fragment>
</CodeMirrorRainlang>
