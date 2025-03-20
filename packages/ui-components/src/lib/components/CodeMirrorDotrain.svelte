<script lang="ts">
	import CodeMirror from 'svelte-codemirror-editor';
	import { RawRainlangExtension } from 'codemirror-rainlang';
	import { openLintPanel } from '@codemirror/lint';

	export let rainlangText: string | undefined = undefined;
	export let disabled = false;
	export let styles = {};
	export let rainlangExtension: RawRainlangExtension;
	export let codeMirrorTheme;
	export let onTextChange: (text: string) => void;
</script>

<div data-testid="codemirror-dotrain">
	<CodeMirror
		value={rainlangText || ''}
		extensions={[rainlangExtension]}
		theme={codeMirrorTheme}
		readonly={disabled}
		useTab={true}
		tabSize={2}
		styles={{
			'&': {
				width: '100%'
			},
			...styles
		}}
		on:change={(e) => {
			onTextChange(e.detail);
		}}
		on:ready={(e) => {
			openLintPanel(e.detail);
		}}
	/>
</div>

<style global>
	:global(.ͼ1 .cm-panel.cm-panel-lint ul [aria-selected]) {
		background-color: inherit;
	}
</style>
