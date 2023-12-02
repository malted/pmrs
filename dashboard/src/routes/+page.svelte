<script lang="ts">
    import Pill from "$lib/components/Pill.svelte";
     import { onDestroy } from "svelte";

    export let data: any;

    enum ViewType { List, Grid }
    let vt: ViewType = ViewType.List;

    const interval = setInterval(async () => {
		data = await fetch("http://localhost:4173/api/services").then(res => res.json())
	}, 1_000);

    onDestroy(() => clearInterval(interval));
</script>

<section data-view={vt === ViewType.List ? "list" : "grid"}>
	<h1 class="text-3xl">Services</h1>

	<button class="p-2 rounded border" on:click={() => vt = ViewType.Grid}>Grid</button>
	<button class="p-2 rounded border" on:click={() => vt = ViewType.List}>List</button>

	<ul class="flex flex-col gap-4">
	{#each data.services as service}
		{@const cfg = service.configuration}
		{@const port = cfg.envs.find((env: [string, string]) => env[0] === "PORT")?.[1]}
		<li class="p-2 rounded-md border">
			<div id="l1-info" class="flex">
				<div class="flex gap-2">
					<h2 class="text-xl">{cfg.name}</h2>
					<Pill type={service.running ? "success" : "error"}>{service.running ? "Running" : "Stopped"}</Pill>
				</div>
				<div class="flex items-center gap-1 rounded ring-1 ring-zinc-300 w-fit p-1 py-0 bg-zinc-200">
					🔄
					<p>{service.restarts}</p>
				</div>
			</div>
			{#if port}
				<div class="flex items-center gap-2">
					🌐 <p>{port}</p> → <a href="https://malted.dev/api">malted.dev/api/</a>
				</div>
			{/if}
		</li>
	{/each}
	</ul>
</section>

<style>
	section {

	}
	section[data-view="grid"] > ul {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(20rem, 1fr));
		grid-gap: 1rem;
	}
	section[data-view="grid"] > ul li #l1-info {
		flex-direction: column;
	}

	    circle {
      transition: fill-opacity 0.2s ease;
      fill-opacity: 0.6;
    }
    circle.selected { fill-opacity: 1; }
</style>