import { createTheme } from 'thememirror';
import { tags } from '@lezer/highlight';

export const lightTheme = createTheme({
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
});