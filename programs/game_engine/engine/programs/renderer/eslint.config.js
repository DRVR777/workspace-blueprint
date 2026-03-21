import js from '@eslint/js'
import tseslint from 'typescript-eslint'
import reactHooks from 'eslint-plugin-react-hooks'

export default tseslint.config(
  js.configs.recommended,
  // Use strict (not typeChecked) — typeChecked requires parserOptions.project and
  // produces false positives for R3F's imperative camera/scene mutation pattern.
  ...tseslint.configs.strict,
  {
    plugins: {
      'react-hooks': reactHooks,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      // R3F pattern: useThree() returns camera/scene refs you ARE supposed to mutate.
      'react-hooks/immutability': 'off',
      // Three.js disposal APIs are typed as any — warn, don't error.
      '@typescript-eslint/no-explicit-any': 'warn',
      // Variables/params prefixed with _ are intentionally unused (interface stubs, future args).
      '@typescript-eslint/no-unused-vars': ['error', {
        varsIgnorePattern: '^_',
        argsIgnorePattern: '^_',
        caughtErrorsIgnorePattern: '^_',
      }],
    },
  },
  {
    ignores: ['dist/', 'node_modules/'],
  },
)
