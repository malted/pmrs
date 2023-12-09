let rc = 0;

const handler = (request: Request): Response => {
  const body = `${rc++}\nYour user-agent is:\n\n${
    request.headers.get("user-agent") ?? "Unknown"
  }`;

  if (rc > 10) Deno.exit(0);

  return new Response(body, { status: 200 });
};

const port = parseInt(Deno.env.get("PORT")!);
console.log(`HTTP server running. Access it at: http://localhost:${port}/`);
Deno.serve({ port }, handler);

//
// Deno.serve(
//   { port: parseInt(Deno.env.get("stienarsteinsrteisro")!) },
//   (a, b) => {
//     console.log(a, b);
//     return new Response("Hello World");
//   },
// );
