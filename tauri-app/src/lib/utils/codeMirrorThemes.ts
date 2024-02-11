import { createTheme } from 'thememirror';
import { tags } from '@lezer/highlight';

export const themes = {
	// configured to resemble vscode default light theme
	light: createTheme({
		variant: 'light',
		settings: {
			background: '#ffffff',
			foreground: '#001080',
			caret: '#000000',
			selection: '#add6ff',
			lineHighlight: '#dadada',
			gutterBackground: '#eeeeee',
			gutterForeground: '#237893'
		},
		styles: [
			{
				tag: tags.comment,
				color: '#008001'
			},
			{
				tag: tags.variableName,
				color: '#0070c1'
			},
			{
				tag: [tags.string, tags.special(tags.brace)],
				color: '#a31515'
			},
			{
				tag: tags.number,
				color: '#00b97b'
			},
			{
				tag: tags.bool,
				color: '#0000ff'
			},
			{
				tag: tags.null,
				color: '#0000ff'
			},
			{
				tag: tags.unit,
				color: '#0000ff'
			},
			{
				tag: tags.keyword,
				color: '#af01db'
			},
			{
				tag: tags.operator,
				color: '#000000'
			},
			{
				tag: tags.className,
				color: '#257f99'
			},
			{
				tag: tags.meta,
				color: '#0950a9'
			},
			{
				tag: tags.definition(tags.typeName),
				color: '#257f99'
			},
			{
				tag: tags.angleBracket,
				color: '#213CF1'
			},
			{
				tag: tags.brace,
				color: '#213CF1'
			},
			{
				tag: tags.bracket,
				color: '#213CF1'
			},
			{
				tag: tags.squareBracket,
				color: '#213CF1'
			},
			{
				tag: tags.paren,
				color: '#213CF1'
			},
			{
				tag: tags.punctuation,
				color: '#464646'
			},
			{
				tag: tags.separator,
				color: '#464646'
			},
			{
				tag: tags.typeName,
				color: '#257f99'
			},
			{
				tag: tags.tagName,
				color: '#800000'
			},
			{
				tag: tags.attributeName,
				color: '#eb3d36'
			}
		]
	}),

	// configured to resemble vscode default dark theme
	dark: createTheme({
		variant: 'dark',
		settings: {
			background: '#1e1e1e',
			foreground: '#d4d4d4',
			caret: '#d4d4d4',
			selection: '#5A7590',
			lineHighlight: '#3c3c41',
			gutterBackground: '#282828',
			gutterForeground: '#858585'
		},
		styles: [
			{
				tag: tags.comment,
				color: '#6c9e57'
			},
			{
				tag: tags.variableName,
				color: '#9cdcfe'
			},
			{
				tag: [tags.string, tags.special(tags.brace)],
				color: '#ce9178'
			},
			{
				tag: tags.number,
				color: '#B6CFA9'
			},
			{
				tag: tags.bool,
				color: '#4fc4ff'
			},
			{
				tag: tags.null,
				color: '#4fc4ff'
			},
			{
				tag: tags.unit,
				color: '#608FC2'
			},
			{
				tag: tags.keyword,
				color: '#d18dcc'
			},
			{
				tag: tags.operator,
				color: '#d4d4d4'
			},
			{
				tag: tags.className,
				color: '#4dcab1'
			},
			{
				tag: tags.meta,
				color: '#608FC2'
			},
			{
				tag: tags.definition(tags.typeName),
				color: '#4fcfb5'
			},
			{
				tag: tags.angleBracket,
				color: '#F9D849'
			},
			{
				tag: tags.brace,
				color: '#F9D849'
			},
			{
				tag: tags.bracket,
				color: '#F9D849'
			},
			{
				tag: tags.squareBracket,
				color: '#F9D849'
			},
			{
				tag: tags.paren,
				color: '#F9D849'
			},
			{
				tag: tags.punctuation,
				color: '#d4d4d4'
			},
			{
				tag: tags.separator,
				color: '#d4d4d4'
			},
			{
				tag: tags.typeName,
				color: '#4ecdb4'
			},
			{
				tag: tags.tagName,
				color: '#59a3df'
			},
			{
				tag: tags.attributeName,
				color: '#a0e1ff'
			}
		]
	})
};