import { json } from "@sveltejs/kit";

export async function GET({ fetch }) {
	try {
		return json({
			success: true,
			services: await fetch("http://localhost:8000/services").then((res) => res.json()),
		});
	} catch (_) {
		return json({ success: false, services: [] });
	}
}
