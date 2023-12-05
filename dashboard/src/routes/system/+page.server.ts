export async function load({ fetch }) {
	return await fetch("/api/system").then((r) => r.json());
}
