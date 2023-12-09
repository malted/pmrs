import { json } from "@sveltejs/kit";

export async function GET({ fetch, url }) {
	console.log(url);

	try {
		const payloadData = await fetch("http://localhost:8000/services");
		const payload = await payloadData.json();

		return json({
			success: true,
			payload
		});
	} catch (_) {
		return json({ success: false, payload: null });
	}
}
