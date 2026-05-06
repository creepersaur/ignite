import Prism from "prismjs";
import { Copy } from "lucide-react";
import "./code.css";
import "./bashLanguage";
import "./igniteLanguage";

export default function CodeBlock({ code, language }) {
	const grammar = Prism.languages[language] ?? Prism.languages.markup;

	const lines = code.split("\n").map((line, i) => {
		const html = Prism.highlight(line, grammar, language);
		const replaced = html.replaceAll(
			"\t",
			'<span class="token tab">\t</span>',
		);

		return (
			<div key={i} className="code-line">
				<span className="line-number">{i + 1}</span>
				<span dangerouslySetInnerHTML={{ __html: replaced }} />
			</div>
		);
	});

	return (
		<div className="code-wrapper" data-language={language}>
			<pre className={`language-${language}`}>
				{lines}
			</pre>
			<button
				className="copy-btn"
				onClick={(event) => {
					navigator.clipboard.writeText(code);
					event.target.setAttribute("copied", true);
					setTimeout(() => event.target.removeAttribute("copied"), 1000);
				}}
			>
				<Copy size={16} color="#5e5e5e" />
			</button>
		</div>
	);
}
