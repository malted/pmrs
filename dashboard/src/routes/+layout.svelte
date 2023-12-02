<script>
	import "$lib/app.css";
	import Navbar from "$lib/components/Navbar.svelte";
</script>

<Navbar />
<main class="m-2">
	<slot />
</main>