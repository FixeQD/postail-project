module.exports = {
	printWidth: 100,
	tabWidth: 4,
	useTabs: true,
	singleQuote: true,
	jsxSingleQuote: true,
	semi: false,
	bracketSameLine: true,
	trailingComma: 'es5',
	arrowParens: 'always',
	endOfLine: 'lf',
	proseWrap: 'preserve',
	quoteProps: 'consistent',
	plugins: ['prettier-plugin-tailwindcss'],
	tailwindAttributes: ['className'],
	overrides: [
		{
			files: ['*.json', '*.jsonc'],
			options: {
				trailingComma: 'none',
			},
		},
	],
}
