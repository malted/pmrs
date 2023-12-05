<script lang="ts">
	export let type: "success" | "error" | "neutral" = "neutral";

	let col: string = "zinc";

	$: switch (type) {
		case "success":
			col = "green";
			break;
		case "error":
			col = "red";
			break;
		case "neutral":
			col = "zinc";
			break;
	}
</script>

<!-- ring-zinc-300 w-fit p-1 py-0 bg-zinc-200 -->

<!-- text-green-50 text-green-900 bg-green-700/50 ring-green-700 -->
<!-- text-red-50 bg-red-700/50 ring-red-700 -->
<!-- text-zinc-50 bg-zinc-700/50 ring-zinc-700 -->
<span class={`inline-flex items-center w-fit rounded-md px-2 py-1 text-xs font-medium bg-${col}-700/50 text-${col}-50 ring-1 ring-inset ring-${col}-700`}><slot /></span>
