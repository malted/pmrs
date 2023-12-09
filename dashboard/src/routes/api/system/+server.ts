import { json } from "@sveltejs/kit";

export async function GET({ fetch }) {
	try {
		return json({
			success: true,
			payload: await fetch("http://localhost:8000/system").then((res) => res.json())
		});
	} catch (_) {
		return json({ success: false, payload: null });
	}
}
