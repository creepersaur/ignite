import { BrowserRouter, Routes, Route } from "react-router-dom";
import Home from "./pages/Home";
import Api from "./pages/Api";
import Docs from "./pages/Docs";

import "./App.css"

export default function App() {
	return (
		<BrowserRouter basename="/ignite">
			<Routes>
				<Route path="/" element={<Home />} />
				<Route path="/docs" element={<Docs />} />
				<Route path="/api" element={<Api />} />
			</Routes>
		</BrowserRouter>
	);
}
