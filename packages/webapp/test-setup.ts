import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';

vi.mock('@reown/appkit', () => ({
	default: vi.fn()
}));
