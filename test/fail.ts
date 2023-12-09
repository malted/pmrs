console.log("hi");

setTimeout(() => {
  try {
    // Simulate an error
    throw new Error("The error we want occurred.");
  } catch (e) {
    console.error(e.message);
    Deno.exit(Math.floor(Math.random() * 255));
  }
}, 2000);
