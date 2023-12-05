export async function load({ fetch }) {
	return await fetch("/api/services").then((r) => r.json());
}
