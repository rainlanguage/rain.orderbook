import { c as create_ssr_component, l as createEventDispatcher, p as onDestroy, h as escape, f as add_attribute } from "./ssr.js";
import { basicSetup } from "codemirror";
import { EditorView, keymap, placeholder } from "@codemirror/view";
import { EditorState } from "@codemirror/state";
import { indentWithTab } from "@codemirror/commands";
import { indentUnit } from "@codemirror/language";
const css = {
  code: ".codemirror-wrapper.svelte-nofj9o .cm-focused{outline:none}.scm-waiting.svelte-nofj9o{position:relative}.scm-waiting__loading.svelte-nofj9o{position:absolute;top:0;left:0;bottom:0;right:0;background-color:rgba(255, 255, 255, 0.5)}.scm-loading.svelte-nofj9o{display:flex;align-items:center;justify-content:center}.scm-loading__spinner.svelte-nofj9o{width:1rem;height:1rem;border-radius:100%;border:solid 2px #000;border-top-color:transparent;margin-right:0.75rem;animation:svelte-nofj9o-spin 1s linear infinite}.scm-loading__text.svelte-nofj9o{font-family:sans-serif}.scm-pre.svelte-nofj9o{font-size:0.85rem;font-family:monospace;-o-tab-size:2;tab-size:2;-moz-tab-size:2;resize:none;pointer-events:none;-webkit-user-select:none;-moz-user-select:none;user-select:none;overflow:auto}@keyframes svelte-nofj9o-spin{0%{transform:rotate(0deg)}100%{transform:rotate(360deg)}}",
  map: '{"version":3,"file":"CodeMirror.svelte","sources":["CodeMirror.svelte"],"sourcesContent":["<script context=\\"module\\"><\/script>\\n\\n<script>import { createEventDispatcher, onDestroy, onMount } from \\"svelte\\";\\nimport { basicSetup } from \\"codemirror\\";\\nimport { EditorView, keymap, placeholder as placeholderExt } from \\"@codemirror/view\\";\\nimport { EditorState, StateEffect } from \\"@codemirror/state\\";\\nimport { indentWithTab } from \\"@codemirror/commands\\";\\nimport { indentUnit } from \\"@codemirror/language\\";\\nimport { debounce } from \\"./util\\";\\nlet classes = \\"\\";\\nexport { classes as class };\\nexport let value = \\"\\";\\nexport let basic = true;\\nexport let lang = void 0;\\nexport let theme = void 0;\\nexport let extensions = [];\\nexport let useTab = true;\\nexport let tabSize = 2;\\nexport let styles = void 0;\\nexport let lineWrapping = false;\\nexport let editable = true;\\nexport let readonly = false;\\nexport let placeholder = void 0;\\nexport let nodebounce = false;\\nconst is_browser = typeof window !== \\"undefined\\";\\nconst dispatch = createEventDispatcher();\\nlet element;\\nlet view;\\nlet update_from_prop = false;\\nlet update_from_state = false;\\nlet first_config = true;\\nlet first_update = true;\\n$: state_extensions = [\\n  ...get_base_extensions(basic, useTab, tabSize, lineWrapping, placeholder, editable, readonly, lang),\\n  ...get_theme(theme, styles),\\n  ...extensions\\n];\\n$: view && update(value);\\n$: view && state_extensions && reconfigure();\\n$: on_change = nodebounce ? handle_change : debounce(handle_change, 300);\\nonMount(() => {\\n  view = create_editor_view();\\n  dispatch(\\"ready\\", view);\\n});\\nonDestroy(() => view?.destroy());\\nfunction create_editor_view() {\\n  return new EditorView({\\n    parent: element,\\n    state: create_editor_state(value),\\n    dispatch(transaction) {\\n      view.update([transaction]);\\n      if (!update_from_prop && transaction.docChanged) {\\n        on_change();\\n      }\\n    }\\n  });\\n}\\nfunction reconfigure() {\\n  if (first_config) {\\n    first_config = false;\\n    return;\\n  }\\n  view.dispatch({\\n    effects: StateEffect.reconfigure.of(state_extensions)\\n  });\\n  dispatch(\\"reconfigure\\", view);\\n}\\nfunction update(value2) {\\n  if (first_update) {\\n    first_update = false;\\n    return;\\n  }\\n  if (update_from_state) {\\n    update_from_state = false;\\n    return;\\n  }\\n  update_from_prop = true;\\n  view.setState(create_editor_state(value2));\\n  update_from_prop = false;\\n}\\nfunction handle_change() {\\n  const new_value = view.state.doc.toString();\\n  if (new_value === value) return;\\n  update_from_state = true;\\n  value = new_value;\\n  dispatch(\\"change\\", value);\\n}\\nfunction create_editor_state(value2) {\\n  return EditorState.create({\\n    doc: value2 ?? void 0,\\n    extensions: state_extensions\\n  });\\n}\\nfunction get_base_extensions(basic2, useTab2, tabSize2, lineWrapping2, placeholder2, editable2, readonly2, lang2) {\\n  const extensions2 = [\\n    indentUnit.of(\\" \\".repeat(tabSize2)),\\n    EditorView.editable.of(editable2),\\n    EditorState.readOnly.of(readonly2)\\n  ];\\n  if (basic2) extensions2.push(basicSetup);\\n  if (useTab2) extensions2.push(keymap.of([indentWithTab]));\\n  if (placeholder2) extensions2.push(placeholderExt(placeholder2));\\n  if (lang2) extensions2.push(lang2);\\n  if (lineWrapping2) extensions2.push(EditorView.lineWrapping);\\n  return extensions2;\\n}\\nfunction get_theme(theme2, styles2) {\\n  const extensions2 = [];\\n  if (styles2) extensions2.push(EditorView.theme(styles2));\\n  if (theme2) extensions2.push(theme2);\\n  return extensions2;\\n}\\n<\/script>\\n\\n{#if is_browser}\\n    <div class=\\"codemirror-wrapper {classes}\\" bind:this={element} />\\n{:else}\\n    <div class=\\"scm-waiting {classes}\\">\\n        <div class=\\"scm-waiting__loading scm-loading\\">\\n            <div class=\\"scm-loading__spinner\\" />\\n            <p class=\\"scm-loading__text\\">Loading editor...</p>\\n        </div>\\n\\n        <pre class=\\"scm-pre cm-editor\\">{value}</pre>\\n    </div>\\n{/if}\\n\\n<style>\\n    .codemirror-wrapper :global(.cm-focused) {\\n        outline: none;\\n    }\\n\\n    .scm-waiting {\\n        position: relative;\\n    }\\n    .scm-waiting__loading {\\n        position: absolute;\\n        top: 0;\\n        left: 0;\\n        bottom: 0;\\n        right: 0;\\n        background-color: rgba(255, 255, 255, 0.5);\\n    }\\n\\n    .scm-loading {\\n        display: flex;\\n        align-items: center;\\n        justify-content: center;\\n    }\\n    .scm-loading__spinner {\\n        width: 1rem;\\n        height: 1rem;\\n        border-radius: 100%;\\n        border: solid 2px #000;\\n        border-top-color: transparent;\\n        margin-right: 0.75rem;\\n        animation: spin 1s linear infinite;\\n    }\\n    .scm-loading__text {\\n        font-family: sans-serif;\\n    }\\n    .scm-pre {\\n        font-size: 0.85rem;\\n        font-family: monospace;\\n        -o-tab-size: 2;\\n           tab-size: 2;\\n        -moz-tab-size: 2;\\n        resize: none;\\n        pointer-events: none;\\n        -webkit-user-select: none;\\n           -moz-user-select: none;\\n                user-select: none;\\n        overflow: auto;\\n    }\\n\\n    @keyframes spin {\\n        0% {\\n            transform: rotate(0deg);\\n        }\\n        100% {\\n            transform: rotate(360deg);\\n        }\\n    }\\n</style>\\n"],"names":[],"mappings":"AAgII,iCAAmB,CAAS,WAAa,CACrC,OAAO,CAAE,IACb,CAEA,0BAAa,CACT,QAAQ,CAAE,QACd,CACA,mCAAsB,CAClB,QAAQ,CAAE,QAAQ,CAClB,GAAG,CAAE,CAAC,CACN,IAAI,CAAE,CAAC,CACP,MAAM,CAAE,CAAC,CACT,KAAK,CAAE,CAAC,CACR,gBAAgB,CAAE,KAAK,GAAG,CAAC,CAAC,GAAG,CAAC,CAAC,GAAG,CAAC,CAAC,GAAG,CAC7C,CAEA,0BAAa,CACT,OAAO,CAAE,IAAI,CACb,WAAW,CAAE,MAAM,CACnB,eAAe,CAAE,MACrB,CACA,mCAAsB,CAClB,KAAK,CAAE,IAAI,CACX,MAAM,CAAE,IAAI,CACZ,aAAa,CAAE,IAAI,CACnB,MAAM,CAAE,KAAK,CAAC,GAAG,CAAC,IAAI,CACtB,gBAAgB,CAAE,WAAW,CAC7B,YAAY,CAAE,OAAO,CACrB,SAAS,CAAE,kBAAI,CAAC,EAAE,CAAC,MAAM,CAAC,QAC9B,CACA,gCAAmB,CACf,WAAW,CAAE,UACjB,CACA,sBAAS,CACL,SAAS,CAAE,OAAO,CAClB,WAAW,CAAE,SAAS,CACtB,WAAW,CAAE,CAAC,CACX,QAAQ,CAAE,CAAC,CACd,aAAa,CAAE,CAAC,CAChB,MAAM,CAAE,IAAI,CACZ,cAAc,CAAE,IAAI,CACpB,mBAAmB,CAAE,IAAI,CACtB,gBAAgB,CAAE,IAAI,CACjB,WAAW,CAAE,IAAI,CACzB,QAAQ,CAAE,IACd,CAEA,WAAW,kBAAK,CACZ,EAAG,CACC,SAAS,CAAE,OAAO,IAAI,CAC1B,CACA,IAAK,CACD,SAAS,CAAE,OAAO,MAAM,CAC5B,CACJ"}'
};
const CodeMirror = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let { class: classes = "" } = $$props;
  let { value = "" } = $$props;
  let { basic = true } = $$props;
  let { lang = void 0 } = $$props;
  let { theme = void 0 } = $$props;
  let { extensions = [] } = $$props;
  let { useTab = true } = $$props;
  let { tabSize = 2 } = $$props;
  let { styles = void 0 } = $$props;
  let { lineWrapping = false } = $$props;
  let { editable = true } = $$props;
  let { readonly = false } = $$props;
  let { placeholder: placeholder$1 = void 0 } = $$props;
  let { nodebounce = false } = $$props;
  const is_browser = typeof window !== "undefined";
  createEventDispatcher();
  let element;
  let view;
  onDestroy(() => view?.destroy());
  function get_base_extensions(basic2, useTab2, tabSize2, lineWrapping2, placeholder2, editable2, readonly2, lang2) {
    const extensions2 = [
      indentUnit.of(" ".repeat(tabSize2)),
      EditorView.editable.of(editable2),
      EditorState.readOnly.of(readonly2)
    ];
    if (basic2) extensions2.push(basicSetup);
    if (useTab2) extensions2.push(keymap.of([indentWithTab]));
    if (placeholder2) extensions2.push(placeholder(placeholder2));
    if (lang2) extensions2.push(lang2);
    if (lineWrapping2) extensions2.push(EditorView.lineWrapping);
    return extensions2;
  }
  function get_theme(theme2, styles2) {
    const extensions2 = [];
    if (styles2) extensions2.push(EditorView.theme(styles2));
    if (theme2) extensions2.push(theme2);
    return extensions2;
  }
  if ($$props.class === void 0 && $$bindings.class && classes !== void 0) $$bindings.class(classes);
  if ($$props.value === void 0 && $$bindings.value && value !== void 0) $$bindings.value(value);
  if ($$props.basic === void 0 && $$bindings.basic && basic !== void 0) $$bindings.basic(basic);
  if ($$props.lang === void 0 && $$bindings.lang && lang !== void 0) $$bindings.lang(lang);
  if ($$props.theme === void 0 && $$bindings.theme && theme !== void 0) $$bindings.theme(theme);
  if ($$props.extensions === void 0 && $$bindings.extensions && extensions !== void 0) $$bindings.extensions(extensions);
  if ($$props.useTab === void 0 && $$bindings.useTab && useTab !== void 0) $$bindings.useTab(useTab);
  if ($$props.tabSize === void 0 && $$bindings.tabSize && tabSize !== void 0) $$bindings.tabSize(tabSize);
  if ($$props.styles === void 0 && $$bindings.styles && styles !== void 0) $$bindings.styles(styles);
  if ($$props.lineWrapping === void 0 && $$bindings.lineWrapping && lineWrapping !== void 0) $$bindings.lineWrapping(lineWrapping);
  if ($$props.editable === void 0 && $$bindings.editable && editable !== void 0) $$bindings.editable(editable);
  if ($$props.readonly === void 0 && $$bindings.readonly && readonly !== void 0) $$bindings.readonly(readonly);
  if ($$props.placeholder === void 0 && $$bindings.placeholder && placeholder$1 !== void 0) $$bindings.placeholder(placeholder$1);
  if ($$props.nodebounce === void 0 && $$bindings.nodebounce && nodebounce !== void 0) $$bindings.nodebounce(nodebounce);
  $$result.css.add(css);
  [
    ...get_base_extensions(basic, useTab, tabSize, lineWrapping, placeholder$1, editable, readonly, lang),
    ...get_theme(theme, styles),
    ...extensions
  ];
  return `${is_browser ? `<div class="${"codemirror-wrapper " + escape(classes, true) + " svelte-nofj9o"}"${add_attribute("this", element, 0)}></div>` : `<div class="${"scm-waiting " + escape(classes, true) + " svelte-nofj9o"}"><div class="scm-waiting__loading scm-loading svelte-nofj9o" data-svelte-h="svelte-1pxusly"><div class="scm-loading__spinner svelte-nofj9o"></div> <p class="scm-loading__text svelte-nofj9o">Loading editor...</p></div> <pre class="scm-pre cm-editor svelte-nofj9o">${escape(value)}</pre></div>`}`;
});
export {
  CodeMirror as C
};
//# sourceMappingURL=CodeMirror.js.map
