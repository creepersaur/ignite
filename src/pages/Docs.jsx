import { useParams } from "react-router-dom";
import { renderToStaticMarkup } from "react-dom/server";
import { useEffect, useRef, useState } from "react";
import Sidebar from "../components/sidebar/sidebar";
import TopBar from "../components/topbar/topbar";
import ReactMarkdown from "react-markdown";
import rehypeRaw from "rehype-raw";
import "./styles/docs.css";
import "../components/doc_components/doc_component.css";
import Overview from "../components/overview/overview";
import "../components/doc_components/code_block";
import CodeBlock from "../components/doc_components/code_block";
import { Link } from "lucide-react";

export default function Docs() {
	const { "*": docPath } = useParams();
	const [content, setContent] = useState("← Select a doc from the left");
	const [headings, setHeadings] = useState([]);
	const contentRef = useRef();

	useEffect(() => {
		if (docPath.length > 0) {
			setContent("Loading...");
			fetch(`/ignite/docs_raw/${docPath}.md`)
				.then((res) => res.text())
				.then(setContent)
				.catch(() => setContent("# Page not found"));
		}
	}, [docPath]);

	useEffect(() => {
		if (!contentRef.current) return;
		contentRef.current.scrollTo(0, 0);

		const nodes = contentRef.current.querySelectorAll("h1, h2, h3");
		const collected = Array.from(nodes).map((el) => {
			const id = el.textContent.toLowerCase().replace(/\s+/g, "-");
			el.id = id;

			const item = {
				text: el.textContent,
				level: Number(el.tagName[1]),
				element: el,
				id,
			};

			const header_link = document.createElement("a");
			header_link.className = "header-link";
			header_link.href = `#${id}`;
			header_link.innerHTML = renderToStaticMarkup(<Link size={16} />);

			el.querySelectorAll(".header-link").forEach((a) => a.remove());
			el.appendChild(header_link);
			el.addEventListener("click", () => {
				const url =
					`${window.location.origin}${window.location.pathname}#${id}`;
				navigator.clipboard.writeText(url);
			}, { once: false });

			if (window.location.hash == `#${id}`) {
				el.scrollIntoView({
					behavior: "smooth",
					block: "start",
				});
			}

			return item;
		});
		setHeadings(collected);
	}, [content]);

	return (
		<>
			<TopBar />

			<div className="docs-main">
				<Sidebar />
				<div className="doc-content" ref={contentRef}>
					<ReactMarkdown
						rehypePlugins={[rehypeRaw]}
						components={{
							code({ inline, className, children }) {
								const match = /language-(\w+)/.exec(
									className || "",
								);
								return !inline && match
									? (
										<CodeBlock
											code={String(children).trim()}
											language={match[1]}
										/>
									)
									: (
										<code className={className}>
											{children}
										</code>
									);
							},
						}}
					>
						{content}
					</ReactMarkdown>
				</div>
				<Overview headings={headings} />
			</div>
		</>
	);
}
