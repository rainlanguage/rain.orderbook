import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';

vi.mock('codemirror-rainlang', () => ({
	RainlangLR: vi.fn()
}));

