<script lang="ts">
	import { onMount } from "svelte";
	import { tweened } from "svelte/motion";
	import { cubicOut } from "svelte/easing";
	import { padStartNbsp } from "$lib/utils";

	export let label: string;
	export let sublabel: string;
	export let value: any;
	export let max: number;
	export let type: "small" | "regular" = "regular";

	let mounted = false;
	onMount(() => (mounted = true));

	$: dpCount = type === "small" ? 0 : 2;

	$: tweenedValue = tweened(0, { duration: 1_000, easing: cubicOut });
	$: if (mounted && value) {
		tweenedValue.set(value);
	}

	$: mainText = type === "small" ? "text-sm" : "";
	$: subText = type === "small" ? "text-xs" : "text-sm";
	$: height = type === "small" ? "h-2" : "h-3";

	$: numericalValue = padStartNbsp((($tweenedValue / max) * 100).toFixed(dpCount).toString(), 3);
</script>

<p class={`${mainText} font-mono`}>{label}</p>
<p class={`${subText} font-mono`}>{@html sublabel}</p>
<progress class={`bg-zinc-700 ${height} w-16 rounded-sm`} value={$tweenedValue} {max} />
<p class={`${subText} font-mono`}>{@html numericalValue}%</p>