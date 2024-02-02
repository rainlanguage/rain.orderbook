<script lang="ts">
	import { darkTheme } from './themes/dark';
  import { lightTheme } from './themes/light';
	import {
		autocompletion,
		closeBrackets,
		closeBracketsKeymap,
		completionKeymap
	} from '@codemirror/autocomplete';
	import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
	import { defaultHighlightStyle, indentOnInput, syntaxHighlighting } from '@codemirror/language';
	import { searchKeymap } from '@codemirror/search';
	import {
		drawSelection,
		highlightActiveLineGutter,
		highlightSpecialChars,
		keymap,
		lineNumbers
	} from '@codemirror/view';
	import { RainLanguage } from 'codemirror-rainlang';
	import CodeMirror from 'svelte-codemirror-editor';

	const rainlangCodemirror = new RainlangExtension({
		hover: false,
		completion: false,
		initialOpMeta: opMeta
	});

	$: raw && compileDocument();

	/// @see https://codemirror.net/docs/extensions/ for the full list of extensions maintained by CodeMirror

	/// Editing
	const whitespace = [indentOnInput()];
	const editingHelpers = [autocompletion(), closeBrackets(), drawSelection(), history()];
	const editingExtension = [whitespace, editingHelpers];

	/// Presentation
	// const styling = [];
	const presentationFeatures = [highlightSpecialChars()];
	const gutters = [highlightActiveLineGutter(), lineNumbers()];
	// const tooltips = [];
	const presentationExtension = [gutters, presentationFeatures];

	/// Input Handling
	const keymapsExtension = keymap.of([
		...closeBracketsKeymap,
		...completionKeymap,
		...defaultKeymap,
		...historyKeymap,
		...searchKeymap
	]);
	const inputHandlingExtension = [keymapsExtension];

	/// Language
	const languageExtension = [syntaxHighlighting(defaultHighlightStyle, { fallback: true })];

</script>

<div class="h-full flex-grow">
	<CodeMirror
		bind:value={raw}
		readonly={readOnly}
		editable={!readOnly}
		theme={$darkMode ? darkTheme : lightTheme}
		styles={{
			'&': {
				flexGrow: 1,
				height: '100%'
			}
		}}
    useTab={true}
    tabSize={2}
		extensions={[
      RainLanguage(),
			editingExtension,
			presentationExtension,
			inputHandlingExtension,
			languageExtension,
			// primitivesExtension,
			rainlangCodemirror.extension
		]}
	/>
</div>

<style lang="postcss" global>
	.codemirror-wrapper {
		display: flex;
		height: 100%;
	}
</style>