<script lang="ts">
	import CodeMirror from 'svelte-codemirror-editor';
	import { RawRainlangExtension } from 'codemirror-rainlang';
	import { codeMirrorTheme } from '$lib/stores/darkMode';

	export let value: string;
	export let disabled = false;
	export let styles = {};
	export let rainlangExtension: RawRainlangExtension;
</script>

<CodeMirror
	bind:value
	extensions={[rainlangExtension]}
	theme={$codeMirrorTheme}
	readonly={disabled}
	useTab={true}
	tabSize={2}
	styles={{
		"&": {
			width: "100%",
		},
		...styles
	}}
/>