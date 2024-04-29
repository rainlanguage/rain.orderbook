import {render, screen} from '@testing-library/svelte'
import { test } from 'vitest'
import Words from './Words.svelte'
import type { ScenariosAuthoringMeta } from '$lib/typeshare/dotrainOrder'
import { expect } from '$lib/test/matchers'
import userEvent from '@testing-library/user-event'

const authoringMetas: ScenariosAuthoringMeta = {
    'scenario1':
        {
            '0x0123': {
                words: [
                    {word: 'apple', description: 'a fruit'},
                    {word: 'banana', description: 'another fruit'},
                    {word: 'carrot', description: 'a vegetable'}
                ]
            },
            '0x4567': {
                words: [
                    {word: 'dog', description: 'a pet'},
                    {word: 'cat', description: 'another pet'},
                    {word: 'fish', description: 'a pet that lives in water'}
                ]
            }
        },
    'scenario2':
        {
            '0x89ab': {
                words: [
                    {word: 'red', description: 'a color'},
                    {word: 'blue', description: 'another color'},
                    {word: 'green', description: 'yet another color'}
                ]
            },
            '0xcdef': {
                words: [
                    {word: 'house', description: 'a building'},
                    {word: 'car', description: 'a vehicle'},
                    {word: 'tree', description: 'a plant'}
                ]
            }
        },
}

test('shows correct words per scenario', async () => {
    render(Words, { authoringMetas });

    await userEvent.click(screen.getByText('scenario1'));

    const words = screen.getAllByTestId('word');
    expect(words).toHaveLength(6);
    expect(words[0]).toHaveTextContent('apple');
    expect(words[1]).toHaveTextContent('banana');
    expect(words[2]).toHaveTextContent('carrot');
    expect(words[3]).toHaveTextContent('dog');
    expect(words[4]).toHaveTextContent('cat');
    expect(words[5]).toHaveTextContent('fish');

    const pragmas = screen.getAllByTestId('pragma');
    expect(pragmas).toHaveLength(2);
    expect(pragmas[0]).toHaveTextContent('0x0123');
    expect(pragmas[1]).toHaveTextContent('0x4567');

    await userEvent.click(screen.getByText('scenario2'));

    const words2 = screen.getAllByTestId('word');
    expect(words2).toHaveLength(6);
    expect(words2[0]).toHaveTextContent('red');
    expect(words2[1]).toHaveTextContent('blue');
    expect(words2[2]).toHaveTextContent('green');
    expect(words2[3]).toHaveTextContent('house');
    expect(words2[4]).toHaveTextContent('car');
    expect(words2[5]).toHaveTextContent('tree');

    const pragmas2 = screen.getAllByTestId('pragma');
    expect(pragmas2).toHaveLength(2);
    expect(pragmas2[0]).toHaveTextContent('0x89ab');
    expect(pragmas2[1]).toHaveTextContent('0xcdef');
})


