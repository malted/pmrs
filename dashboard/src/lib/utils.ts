export const padStartNbsp = (str: string, targetLength: number) =>
	"&nbsp;".repeat(Math.max(0, targetLength - str.length)) + str;

export const panelClass =
	"rounded-md ring-1 ring-zinc-200 dark:ring-zinc-700 w-fit p-1 bg-zinc-100 dark:bg-zinc-800";

export const base = import.meta.env.PROD ? "/admin/" : "/";

// export function setupWebSocket(
// 	sendMessage: string,
// 	onDataReceived: (data: any) => void
// ): () => void {
// 	let wsInterval: NodeJS.Timeout;

// 	const ws = new WebSocket("ws://localhost:8000/ws");

// 	ws.addEventListener("open", () => {
// 		if (wsInterval) clearInterval(wsInterval);
// 		wsInterval = setInterval(() => ws.send(sendMessage), 1_000);
// 	});

// 	ws.addEventListener("message", (event) => {
// 		onDataReceived(JSON.parse(event.data));
// 	});

// 	ws.addEventListener("close", () => clearInterval(wsInterval));
// 	ws.addEventListener("error", () => clearInterval(wsInterval));

// 	const cleanup = () => {
// 		clearInterval(wsInterval);
// 		ws.close();
// 	};

// 	return cleanup;
// }
