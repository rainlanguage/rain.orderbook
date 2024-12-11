import { mockIPC } from '@tauri-apps/api/mocks';
import '@testing-library/jest-dom/vitest'
import { vi } from 'vitest';

vi.mock('svelte-codemirror-editor', () => ({
	default: vi.fn()
}));

vi.mock('codemirror-rainlang', () => ({
	RainlangLR: {}
}));

// Setup the IPC mock globally
mockIPC(() => {
    // Add your conditional logic for different commands here
    return Promise.resolve();
  });
