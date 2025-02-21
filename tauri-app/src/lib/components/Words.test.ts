import { render, screen } from '@testing-library/svelte';
import { test } from 'vitest';
import Words from './Words.svelte';
import type { ScenarioWords } from '@rainlanguage/orderbook/js_api';
import { expect } from '$lib/test/matchers';
import userEvent from '@testing-library/user-event';

const authoringMetas: ScenarioWords[] = [
  {
    scenario: 'scenario1',
    deployerWords: {
      address: '0x4567',
      words: {
        type: 'Success',
        data: {
          words: [
            { word: 'dog', description: 'an animal' },
            { word: 'cat', description: 'another animal' },
            { word: 'fish', description: 'yet another animal' },
          ],
        },
      },
    },
    pragmaWords: [
      {
        address: '0x0123',
        words: {
          type: 'Success',
          data: {
            words: [
              { word: 'apple', description: 'a fruit' },
              { word: 'banana', description: 'another fruit' },
              { word: 'carrot', description: 'a vegetable' },
            ],
          },
        },
      },
    ],
  },
  {
    scenario: 'scenario2',
    deployerWords: {
      address: '0x4567',
      words: {
        type: 'Success',
        data: {
          words: [
            { word: 'dog', description: 'an animal' },
            { word: 'cat', description: 'another animal' },
            { word: 'fish', description: 'yet another animal' },
          ],
        },
      },
    },
    pragmaWords: [
      {
        address: '0x89ab',
        words: {
          type: 'Success',
          data: {
            words: [
              { word: 'red', description: 'a color' },
              { word: 'blue', description: 'another color' },
              { word: 'green', description: 'yet another color' },
            ],
          },
        },
      },
      {
        address: '0xcdef',
        words: {
          type: 'Success',
          data: {
            words: [
              { word: 'house', description: 'a building' },
              { word: 'car', description: 'a vehicle' },
              { word: 'tree', description: 'a plant' },
            ],
          },
        },
      },
    ],
  },
];

test('shows correct words per scenario', async () => {
  render(Words, { authoringMetas, error: undefined });

  await userEvent.click(screen.getByText('scenario1'));

  const words = screen.getAllByTestId('word');
  expect(words).toHaveLength(6);
  expect(words[0]).toHaveTextContent('dog');
  expect(words[1]).toHaveTextContent('cat');
  expect(words[2]).toHaveTextContent('fish');
  expect(words[3]).toHaveTextContent('apple');
  expect(words[4]).toHaveTextContent('banana');
  expect(words[5]).toHaveTextContent('carrot');

  const pragmas = screen.getAllByTestId('pragma');
  expect(pragmas).toHaveLength(2);
  expect(pragmas[0]).toHaveTextContent('0x4567');
  expect(pragmas[1]).toHaveTextContent('0x0123');

  await userEvent.click(screen.getByText('scenario2'));

  const words2 = screen.getAllByTestId('word');
  expect(words2).toHaveLength(9);
  expect(words2[0]).toHaveTextContent('dog');
  expect(words2[1]).toHaveTextContent('cat');
  expect(words2[2]).toHaveTextContent('fish');
  expect(words2[3]).toHaveTextContent('red');
  expect(words2[4]).toHaveTextContent('blue');
  expect(words2[5]).toHaveTextContent('green');
  expect(words2[6]).toHaveTextContent('house');
  expect(words2[7]).toHaveTextContent('car');
  expect(words2[8]).toHaveTextContent('tree');

  const pragmas2 = screen.getAllByTestId('pragma');
  expect(pragmas2).toHaveLength(3);
  expect(pragmas2[0]).toHaveTextContent('0x4567');
  expect(pragmas2[1]).toHaveTextContent('0x89ab');
  expect(pragmas2[2]).toHaveTextContent('0xcdef');
});

test('shows error message when error is present', async () => {
  render(Words, { authoringMetas: undefined, error: 'Test error' });

  const errorMsg = screen.getByTestId('error-msg');
  expect(errorMsg).toHaveTextContent('Test error');
});

const authoringMetaWithPragmaError: ScenarioWords[] = [
  {
    scenario: 'scenario1',
    deployerWords: {
      address: '0x4567',
      words: {
        type: 'Success',
        data: {
          words: [
            { word: 'dog', description: 'an animal' },
            { word: 'cat', description: 'another animal' },
            { word: 'fish', description: 'yet another animal' },
          ],
        },
      },
    },
    pragmaWords: [
      {
        address: '0x0123',
        words: {
          type: 'Error',
          data: 'Test error',
        },
      },
    ],
  },
];

test('shows error message when error is present in pragma', async () => {
  render(Words, { authoringMetas: authoringMetaWithPragmaError, error: undefined });

  await userEvent.click(screen.getByText('scenario1'));

  const errorMsg = screen.getByTestId('pragma-error-msg');
  expect(errorMsg).toHaveTextContent('Test error');
});

const authoringMetaWithDeployerError: ScenarioWords[] = [
  {
    scenario: 'scenario1',
    deployerWords: {
      address: '0x4567',
      words: {
        type: 'Error',
        data: 'Test error',
      },
    },
    pragmaWords: [],
  },
];

test('shows error message when error is present in deployer', async () => {
  render(Words, { authoringMetas: authoringMetaWithDeployerError, error: undefined });

  await userEvent.click(screen.getByText('scenario1'));

  const errorMsg = screen.getByTestId('deployer-error-msg');
  expect(errorMsg).toHaveTextContent('Test error');
});
