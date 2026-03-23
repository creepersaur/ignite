import { Link } from 'react-router-dom'
import TopBar from "../components/topbar/topbar";
import icon from '../assets/IgniteIcon.svg'
import "./styles/home.css"

export default function Home() {
	return <>
		<TopBar />

		<div className="main">
			<div className="title-header">
				<img src={icon} alt="Ignite Image" />

				<h1>
					<span className="japanese">点火す</span>
					Ignite
				</h1>
			</div>

			<div className="description">
				A dynamically typed bytecode compiled language made
				as a hobby project. <br /> Inspired by Rust, Python, C#, and JS.
			</div>

			<div className="button-holder">
				<Link className="primary" to="/docs">Documentation</Link>
				<Link to="/api">API</Link>
			</div>
		</div>
	</>;
}
