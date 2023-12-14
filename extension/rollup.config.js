// rollup.config.js
import typescript from '@rollup/plugin-typescript';

export default [
    {
        input: 'background/background.ts',
        output: {
            file: 'background/background.js',
            format: 'cjs'
        },
        plugins: [typescript()]
    },
    {
        input: 'popup/popup.ts',
        output: {
            file: 'popup/popup.js',
            format: 'cjs'
        },
        plugins: [typescript()]
    },
    {
        input: 'content_scripts/index.ts',
        output: {
            file: 'content_scripts/index.js',
            format: 'cjs'
        },
        plugins: [typescript()]
    }
];