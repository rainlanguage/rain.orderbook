import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, waitFor } from '@testing-library/svelte';
import { CodeMirrorDotrain } from '@rainlanguage/ui-components';
import { RawRainlangExtension } from 'codemirror-rainlang'; // Direct import
import { writable } from 'svelte/store';

// Mock dependencies
vi.mock('codemirror-rainlang', () => ({
	RawRainlangExtension: vi.fn(() => ({
		plugin: vi.fn(),
		hover: vi.fn(),
		completion: vi.fn(),
		language: vi.fn(),
		diagnostics: vi.fn()
	}))
}));

vi.mock('@codemirror/lint', () => ({
	openLintPanel: vi.fn()
}));

vi.mock('svelte-codemirror-editor', async () => {
	const mockCodeMirror = (await import('../lib/__mocks__/MockComponent.svelte')).default;
	return { default: mockCodeMirror };
});

describe('CodeMirrorDotrain', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('should render codemirror-dotrain', async () => {
		const testValue = 'initial dotrain value';
		const rainlangExtensionMock = new RawRainlangExtension({
			hover: async () => null,
			completion: async () => null,
			diagnostics: async () => []
		});

		const screen = render(CodeMirrorDotrain, {
			props: {
				rainlangText: testValue,
				disabled: false,
				styles: {},
				rainlangExtension: rainlangExtensionMock,
				codeMirrorTheme: writable({})
			}
		});

		await waitFor(() => {
			expect(screen.getByTestId('codemirror-dotrain')).toBeInTheDocument();
		});
	});
});
