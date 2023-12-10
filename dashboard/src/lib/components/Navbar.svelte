<script lang="ts">
	import Logo from "./Logo.svelte";
	import { base } from "$lib/utils";
</script>

<nav class="flex justify-between p-3 bg-zinc-100 dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100">
	<div class="flex items-center gap-4">
		<Logo />
		<a href="{base}/" class="ml-4">Services</a>
		<a href="{base}/system">System</a>
	</div>

	<ul>
		<div class="relative">
			<img
				src="https://github.com/malted.png"
				alt="github avatar"
				class="w-10 rounded-full border border-zinc-300 dark:border-zinc-600"
			/>
			<div
				class="absolute w-5 rounded-full bg-zinc-100 dark:bg-zinc-800 bottom-[-10%] left-[-10%]"
			>
				<img src="{base}/icons/github.svg" alt="github icon" class="dark:invert" />
			</div>
		</div>
	</ul>
</nav>
