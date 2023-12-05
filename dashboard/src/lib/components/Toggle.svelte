<script lang="ts">
	export let checked: boolean;
</script>

<label class="relative inline-block w-12 h-6">
	<input type="checkbox" class="opacity-0 w-0 h-0 peer" bind:checked>
	<span class={`
absolute rounded inset-0 bg-zinc-700 transition 
before:absolute !before:content-none before:h-6 before:w-6 before:left-0 before:bottom-0 before:bg-zinc-500 before:rounded before:transition 
peer-checked:bg-zinc-600 peer-checked:before:translate-x-full
`}></span> 
</label>
