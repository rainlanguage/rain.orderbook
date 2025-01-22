import { render } from '@testing-library/svelte';
import { describe, it } from 'vitest';
import Sidebar from '../lib/components/Sidebar.svelte';
import {writable} from "svelte/store";

describe('Sidebar', () => {
    it('renders correctly with colorTheme store', async () => {
        const mockPage = {
            url: {
                pathname: '/',
            },
        };

        // Create a mock store for colorTheme
        const mockColorTheme = writable('light');

        render(Sidebar, {
            props: {
                colorTheme: mockColorTheme,
                page: mockPage,
            },
        });
    });
});
