import { defineConfig, mergeConfig } from 'vitest/config';
import viteConfig from './vite.config';

export default mergeConfig(viteConfig, defineConfig({
  test: {
    // The custom host renderer needs client SFC functions; SSR-only snapshot
    // tests keep their existing transform and do not need a DOM emulator.
    testTransformMode: { web: ['**/workspaceTools.test.ts', '**/inlineChat.test.ts', '**/dictationControl.test.ts'] },
  },
}));
