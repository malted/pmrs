<script lang="ts">
	import { onMount } from "svelte";
	import ms from "ms";
	import { padStartNbsp, panelClass, setupWebSocket } from "$lib/utils";
	import Progress from "$lib/components/Progress.svelte";

	export let data: any;

	onMount(() => {
        return setupWebSocket("system", (receivedData) => (data.payload = receivedData));
    });

	$: sys = data.payload;
	const cpuLabel = (freq: number) => `${(freq / 1_000).toFixed(1)}<span class="text-zinc-400">GHz</span>`;
	const gbLabel = (bytes: number) => `${padStartNbsp((bytes / 1_000_000_000).toFixed(2), 6)}<span class="text-zinc-400">GB</span>`;
</script>

<section class="w-full flex flex-col items-center gap-4">
	<h2 class="text-3xl">{sys.host_name}</h2>

	<div class="flex flex-col items-center gap-2">
		
		<h3>{sys.long_os_version} ({sys.kernel_version})</h3>
		<p class="text-sm">booted {ms(Date.now() - sys.boot_time * 1_000)} ago</p>
	</div>

	<div class="flex gap-2">
		{#each sys.users as user}
			<p class={`text-xs ${panelClass}`}>{user.name}</p>
		{/each}
	</div>

	<div class="flex items-start gap-3">
		<div class={`grid grid-template-4-autos gap-x-4 items-center ${panelClass}`}>
			<Progress label={sys.global_cpu.brand} sublabel={cpuLabel(sys.global_cpu.frequency)} value={sys.global_cpu.cpu_usage} max={100} />

			{#each sys.cpus as cpu}
				<Progress label={cpu.name} sublabel={cpuLabel(sys.global_cpu.frequency)} value={cpu.cpu_usage} max={100} type="small" />
			{/each}
		</div>

		<div class="flex flex-col gap-3">
			<div class={`grid grid-template-4-autos gap-x-4 items-center ${panelClass}`}>
				<Progress label="Memory" sublabel={gbLabel(sys.mem_used)} value={sys.mem_used} max={sys.mem_total} />
				<Progress label="Swap" sublabel={gbLabel(sys.swap_used)} value={sys.swap_used} max={sys.swap_total} />
			</div>

			<div class={`grid grid-template-4-autos gap-x-4 items-center ${panelClass}`}>
				{#each sys.disks as disk}
					<Progress label={`${disk.name} (${disk.file_system} ${disk.type})`} sublabel={`${gbLabel(disk.total_space - disk.available_space)} / ${gbLabel(disk.total_space)}`} value={disk.total_space - disk.available_space} max={disk.total_space} />
				{/each}
			</div>
		</div>
	</div>
</section>

<style>
	.grid-template-4-autos {
		grid-template-columns: repeat(4, auto);
	}
</style>