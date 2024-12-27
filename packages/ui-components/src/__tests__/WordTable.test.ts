import { render, screen } from '@testing-library/svelte';
import { test, expect } from 'vitest';
import WordTable from '$lib/components/WordTable.svelte';
import userEvent from '@testing-library/user-event';

const authoringMeta = {
	words: [
		{ word: 'apple', description: 'a fruit' },
		{ word: 'banana', description: 'another fruit' },
		{ word: 'carrot', description: 'a vegetable' }
	]
};

test('shows initial words', () => {
	render(WordTable, { authoringMeta, pragma: '0x0123' });

	const words = screen.getAllByTestId('word');
	expect(words).toHaveLength(3);
	expect(words[0]).toHaveTextContent('apple');
	expect(words[1]).toHaveTextContent('banana');
	expect(words[2]).toHaveTextContent('carrot');
});

test('shows initial descriptions', () => {
	render(WordTable, { authoringMeta, pragma: '0x0123' });

	const descriptions = screen.getAllByTestId('description');
	expect(descriptions).toHaveLength(3);
	expect(descriptions[0]).toHaveTextContent('a fruit');
	expect(descriptions[1]).toHaveTextContent('another fruit');
	expect(descriptions[2]).toHaveTextContent('a vegetable');
});

test('shows no words when there are none', () => {
	// this shouldn't really ever happen but if it does we don't want it to break
	render(WordTable, { authoringMeta: { words: [] }, pragma: '0x0123' });

	const words = screen.queryAllByTestId('word');
	expect(words).toHaveLength(0);

	// will show the no results message
	const noneMsg = screen.getByTestId('no-results-msg');
	expect(noneMsg).toHaveTextContent('No words found');
});

test('shows no words when there are no matching words', async () => {
	render(WordTable, { authoringMeta, pragma: '0x0123' });

	const input = screen.getByTestId('search-input');
	await userEvent.type(input, 'z');

	const words = screen.queryAllByTestId('word');
	expect(words).toHaveLength(0);

	const noneMsg = screen.getByTestId('no-results-msg');
	expect(noneMsg).toHaveTextContent('No words found');
});

test('searches words based on input', async () => {
	const user = userEvent.setup();
	render(WordTable, { authoringMeta, pragma: '0x0123' });

	const input = screen.getByTestId('search-input');
	await user.type(input, 'app');

	const words = screen.getAllByTestId('word');
	expect(words).toHaveLength(1);

	expect(words[0]).toHaveTextContent('apple');
});

test('shows the correct pragma', () => {
	render(WordTable, { authoringMeta, pragma: '0x0123' });

	const pragma = screen.getByTestId('pragma');
	expect(pragma).toHaveTextContent('0x0123');
});
