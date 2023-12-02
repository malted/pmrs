import { error } from "@sveltejs/kit";

export async function load({ params }) {
	const getServices = async () =>
		await fetch("http://localhost:8000/services").then((r) => r.json());
	try {
		return { success: true, services: await getServices() };
	} catch (e) {
		return { success: false, services: [] }
	}
}
