<script lang="ts">
	export let type: "success" | "error" = "success";
</script>

{#if type === "success"}
	<span class="inline-flex items-center rounded-md bg-green-50 px-2 py-1 text-xs font-medium text-green-700 ring-1 ring-inset ring-green-600/20"><slot /></span>
{:else}
	<span class="inline-flex items-center rounded-md bg-red-50 px-2 py-1 text-xs font-medium text-red-700 ring-1 ring-inset ring-red-600/10"><slot /></span>
{/if}
