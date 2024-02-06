<script lang="ts">
	import { lightTheme } from '../utils/codeMirrorThemes';
	import CodeMirror from 'svelte-codemirror-editor';
	import { parseDotrain } from '../utils/parseDotrain';
	import { RainlangExtension, type LanguageServicesConfig, RainDocument, type Problem, MetaStore } from 'codemirror-rainlang';

	// @TODO - reactive vars, for fork url and block number
	export let forkUrl: string;
	export let forkBlockNumber: number;
	$: forkUrl;
	$: forkBlockNumber;

	// the fork calback fn
	const callback = async(dotrain: RainDocument): Promise<Problem[]> => {
		return parseDotrain(dotrain, forkUrl, forkBlockNumber);
	}

	// extension config
	const metaStore = new MetaStore(false);
	const config: LanguageServicesConfig = {
		hover: true,
		completion: true,
		callback,
		metaStore
	};
	const rainlangCodemirror = new RainlangExtension(config);

	// in order to get the codemirror EditorView from the plugin,
	// plugin is undefined until the codemirror ext is instantiated/running,
	// for example: open/close problem panel - it also has a default hotkey: cmd + shift + m
	// can be opened right after extension gets running
	// let plugin = rainlangCodemirror.plugin;
	// let editorView = plugin.view;
	// import { openLintPanel, closeLintPanel } from "@codemirror/lint";
	// if (editorView) openLintPanel(editorView)

	// the extension theme, light/dark
	const activeTheme = lightTheme;

	// dotrain string bound to editor
	export let dotrain_string: string;

</script>

<CodeMirror
	bind:value={dotrain_string}
	placeholder="start writing your strategy here..."
	theme={activeTheme}
	styles={{
		'&': {
			flexGrow: 1,
			height: '100%'
		}
	}}
	useTab={true}
	tabSize={2}
	extensions={[
		rainlangCodemirror
	]}
/>
