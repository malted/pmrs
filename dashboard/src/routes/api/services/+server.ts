import { json } from "@sveltejs/kit";

export async function GET({ fetch, url }) {
	console.log(url);

	try {
		return json({
			success: true,
			payload: await fetch("http://localhost:8000/services").then((res) => res.json()),
		});
	} catch (_) {
		return json({ success: false, payload: null });
	}
}
