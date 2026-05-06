import Prism from "prismjs";

Prism.languages.ignite = {
	comment: /\/\/.*/,
	string: {
		pattern: /\$?"(?:[^"\\]|\\.)*"/,
		greedy: true,
	},
	fstring: {
		pattern: /\$"(?:[^"\\{}]|\\.|\{[^}]*\})*"/,
		greedy: true,
		alias: "string",
	},
	keyword:
		/\b(fn|let|const|if|else|loop|while|for|in|return|break|continue|out|struct|enum|class|constructor|using|and|or|not|self)\b/,
	boolean: /\b(true|false|nil)\b/,

	// SCREAMING_SNAKE_CASE
	constant: /\b[A-Z][A-Z_0-9]*\b/,

	// PascalCase = types (e.g. MyClass, Vec, String)
	"class-name": /\b[A-Z][a-zA-Z0-9]*\b/,
	builtin_types: {
		pattern: /\b(number|string|char|bool|list|dict)\b/,
		alias: "class-name",
	},

	// snake_case followed by ( = function call
	function:
		/\b[a-z][a-z0-9]*(?:_[a-z0-9]+)+(?=\s*\()|\b[a-z][a-z0-9]*(?=\s*\()/,

	property: /(?<=\.{?)[a-z_][a-z_0-9]*|(?<=::\{?)[a-z_][a-z_0-9]*/,

	number: /\b\d+(\.\d+)?\b/,
	operator: /[+\-*/=<>!?$]\b/,
	"double-semi": /;;/,
	punctuation: /[{}[\]();,-:.]/,
};
