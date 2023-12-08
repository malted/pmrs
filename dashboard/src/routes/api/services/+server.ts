import { json } from "@sveltejs/kit";
import { page } from "$app/stores";

let currentUrl: URL;
let foo = page.subscribe(({ url }) => (currentUrl = url));

export async function GET({ fetch }) {
	console.log(currentUrl.href);

	try {
		return json({
			success: true,
			payload: await fetch("http://localhost:8000/services").then((res) => res.json()),
		});
	} catch (_) {
		return json({ success: false, payload: null });
	}
}
