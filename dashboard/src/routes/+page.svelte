<script lang="ts">
	import { onMount } from "svelte";
	import { panelClass } from "$lib/utils";
	import Pill from "$lib/components/Pill.svelte";
	import Toggle from "$lib/components/Toggle.svelte";

	export let data: any;

	onMount(() => {
		const interval = setInterval(async () => {
			const res = await fetch("/api/services");
			data.payload = await res.json();
		}, 5_000);

		return () => clearInterval(interval);
	});

	let gridChecked: boolean;
	enum ViewType {
		List,
		Grid
	}
	$: vt = gridChecked ? ViewType.Grid : ViewType.List;
</script>

<section data-view={vt === ViewType.List ? "list" : "grid"}>
	<div class="flex justify-between items-center">
		<h1 class="text-3xl">Services</h1>

		<div class="flex items-center gap-2 my-4 px-1 {panelClass}">
			<p class="select-none">List</p>
			<Toggle bind:checked={gridChecked} />
			<p class="select-none">Grid</p>
		</div>
	</div>

	{#if data.payload}
		<ul class="flex flex-col gap-4">
			{#each data.payload as service}
				{@const cfg = service.configuration}
				{@const port = cfg.envs.find((env) => env[0] === "PORT")?.[1]}
				<li class="{panelClass} w-full">
					<div id="l1-info" class="flex gap-1">
						<div class="flex gap-2">
							<h2 class="text-xl">{cfg.name}</h2>
							<Pill type={service.running ? "success" : "error"}
								>{service.running ? "Running" : "Stopped"}</Pill
							>
						</div>
						<Pill type="neutral">
							🔄&nbsp;
							<p>{service.restarts}</p>
						</Pill>
					</div>
					{#if port}
						<div class="flex items-center gap-2">
							🌐 <p>{port}</p>
							→<a href="https://malted.dev/api">malted.dev/api/</a>
						</div>
					{/if}
				</li>
			{/each}
		</ul>
	{/if}
</section>

<style>
	section[data-view="grid"] > ul {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(20rem, 1fr));
		grid-gap: 1rem;
	}

	circle {
		transition: fill-opacity 0.2s ease;
		fill-opacity: 0.6;
	}
	circle.selected {
		fill-opacity: 1;
	}
</style>
