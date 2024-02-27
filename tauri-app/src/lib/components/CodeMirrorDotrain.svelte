<script lang="ts">
	import CodeMirror from 'svelte-codemirror-editor';
	import { completionCallback, hoverCallback, problemsCallback } from '$lib/services/langServices';
	import { RawRainlangExtension, type RawLanguageServicesCallbacks } from 'codemirror-rainlang';
	import { codeMirrorTheme } from '$lib/stores/darkMode';

	export let value: string;
	export let disabled = false;
	export let styles = {};

	const callbacks: RawLanguageServicesCallbacks = {
		hover: hoverCallback,
		completion: completionCallback,
		diagnostics: problemsCallback
	}
	const rainlangExtension = new RawRainlangExtension(callbacks);
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