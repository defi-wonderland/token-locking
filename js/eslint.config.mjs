import { fixupConfigRules } from '@eslint/compat';
import globals from 'globals';
import babelParser from '@babel/eslint-parser';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import js from '@eslint/js';
import { FlatCompat } from '@eslint/eslintrc';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const compat = new FlatCompat({
  baseDirectory: __dirname,
  recommendedConfig: js.configs.recommended,
  allConfig: js.configs.all
});

export default [{
  ignores: ['lib', 'dist'],
}, ...fixupConfigRules(
  compat.extends('eslint:recommended', 'plugin:import/errors', 'plugin:import/warnings'),
), {
  languageOptions: {
    globals: {
      ...globals.browser,
      ...globals.node,
    },
    parser: babelParser,
    parserOptions: {
      requireConfigFile: false,
    },
    ecmaVersion: 8,
    sourceType: 'module',
  },

  settings: {
    react: {
      version: 'detect',
    },
  },

  rules: {
    'no-trailing-spaces': ['error'],
    'import/first': ['error'],
    'import/no-commonjs': ['error'],
    'import/namespace': ['warn'],

    'import/order': ['error', {
      groups: [['internal', 'external', 'builtin'], ['index', 'sibling', 'parent']],
      'newlines-between': 'always',
    }],

    indent: ['error', 2, {
      MemberExpression: 1,
      SwitchCase: 1,
    }],

    'linebreak-style': ['error', 'unix'],
    'no-console': [0],

    quotes: ['error', 'single', {
      avoidEscape: true,
      allowTemplateLiterals: true,
    }],

    'require-await': ['error'],
    semi: ['error', 'always'],
  },
}];