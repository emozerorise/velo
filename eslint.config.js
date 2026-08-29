import globals from 'globals';
import js from '@eslint/js';
import tseslint from 'typescript-eslint';
import pluginVue from 'eslint-plugin-vue';
import configPrettier from 'eslint-config-prettier';

export default tseslint.config(
  {
    // src-tauri is Rust; cargo fmt and clippy cover it.
    ignores: ['dist', 'node_modules', 'src-tauri', 'coverage'],
  },

  js.configs.recommended,
  tseslint.configs.recommended,
  pluginVue.configs['flat/recommended'],

  {
    files: ['**/*.{ts,vue}'],
    languageOptions: {
      globals: globals.browser,
      parserOptions: {
        // Type information, needed by the promise rules below.
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
        // Single-file components are parsed by vue-eslint-parser, which hands
        // the script block to this one.
        parser: tseslint.parser,
        extraFileExtensions: ['.vue'],
      },
    },
    rules: {
      // The reason this config exists. A rejected `invoke` on an un-awaited
      // store action disappears silently, which reads as "the button does
      // nothing" -- the exact failure this project spent a day chasing.
      '@typescript-eslint/no-floating-promises': 'error',
      '@typescript-eslint/no-misused-promises': [
        'error',
        // Vue template handlers are void-returning by design; an async
        // handler there is idiomatic, not a bug.
        { checksVoidReturn: false },
      ],
    },
  },

  {
    // Tooling that runs under Node, not in the webview.
    files: ['tests/**/*.ts', 'scripts/**/*.mjs', '*.config.js'],
    languageOptions: {
      globals: globals.node,
    },
  },

  // Last: drops the stylistic rules that would fight Prettier.
  configPrettier
);
