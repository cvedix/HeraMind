module.exports = {
  root: true,
  env: { browser: true, es2020: true },
  extends: [
    'eslint:recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:react-hooks/recommended',
  ],
  ignorePatterns: ['dist', '.eslintrc.cjs', 'e2e'],
  parser: '@typescript-eslint/parser',
  plugins: ['react-refresh'],
  rules: {
    'react-refresh/only-export-components': [
      'warn',
      { allowConstantExport: true },
    ],
    '@typescript-eslint/no-explicit-any': 'warn',
    '@typescript-eslint/no-unused-vars': ['warn', {
      argsIgnorePattern: '^_',
      varsIgnorePattern: '^_'
    }],
    // Allow console.log in development, but warn in production
    'no-console': process.env.NODE_ENV === 'production'
      ? ['error', { allow: ['warn', 'error'] }]
      : 'off',
    // Disable problematic rules
    'no-unexpected-multiline': 'off',
    'no-case-declarations': 'off',
    'no-extra-semi': 'off',  // Leading semicolons are needed for type assertions in switch cases
    '@typescript-eslint/no-var-requires': 'off',
    '@typescript-eslint/triple-slash-reference': 'off',
  },
  // Ignore test console statements
  overrides: [
    {
      files: ['e2e/**/*.{ts,tsx}', '**/*.spec.{ts,tsx}'],
      rules: {
        'no-console': 'off',
      },
    },
  ],
}
