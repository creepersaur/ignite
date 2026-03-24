import TopBar from "../components/topbar/topbar";
import "./styles/not_found.css"

export default function NotFound() {
	return <>
		<TopBar/>

		<div className="main">
			<h1>404</h1>
			Maybe you linked to the wrong page :(
		</div>
	</>
}