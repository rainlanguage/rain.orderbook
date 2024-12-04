import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render } from '@testing-library/svelte';
import CodeMirrorRainlang from '../lib/components/CodeMirrorRainlang.svelte';
import type { Order } from '@rainlanguage/orderbook/js_api';
import * as orderBookApi from '@rainlanguage/orderbook/js_api';
import { writable } from 'svelte/store';

// Mock the extendOrder function
vi.mock('@rainlanguage/orderbook/js_api', () => ({
  extendOrder: vi.fn((order: Order) => ({
    rainlang: 'mocked rainlang text'
  }))
}));



describe('CodeMirrorRainlang', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should use extendOrder when order prop is provided', () => {
    const mockOrder: Order = {} as Order

    render(CodeMirrorRainlang, {

        order: mockOrder,
        rainlangText: 'original text',
        codeMirrorTheme: writable({})

    });

    expect(orderBookApi.extendOrder).toHaveBeenCalledWith(mockOrder);
  });

  it('should use rainlangText when no order is provided', () => {
    const testText = 'test rainlang text';

    render(CodeMirrorRainlang, {

        rainlangText: testText,
        codeMirrorTheme: writable({})

    });

    expect(orderBookApi.extendOrder).not.toHaveBeenCalled();
  });

  it('should respect disabled prop', () => {
    const { container } = render(CodeMirrorRainlang, {

        disabled: true,
        rainlangText: 'test',
        codeMirrorTheme: writable({})

    });

    const editor = container.querySelector('.cm-editor');
    expect(editor?.getAttribute('contenteditable')).toBe('false');
  });
});
