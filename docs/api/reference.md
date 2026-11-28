# API reference

Every operation of the opengrowth API, with its requests, responses, and schemas, from the OpenAPI document [openapi.json](openapi.json). The document is generated from opengrowth's own code by `opengrowth openapi`, so it lists exactly the operations the server has, and a test checks that it differs from the RCPCH Digital Growth Charts API's document only in opengrowth's name, its provenance, its token, and one correction: upstream's document declares each sex's chart lines as an object, where both servers send `null` for the sex not asked for. Another test replays every successful request captured from the RCPCH server and checks that opengrowth's response matches the document. The API's own [differences](../differences.md) from the RCPCH server are listed separately.

To publish the document with your own server's address, generate it with that address:

```sh
opengrowth openapi --server https://growth.example.org > openapi.json
```

The server does not serve the document itself: its `/` is a health check, where the RCPCH server returns its document.

Below, "Try it out" sends requests to the server chosen in the Servers list: the public [demo server](../integrating/index.md#try-the-demo-server), unless you choose `opengrowth serve` on your own computer, which your browser may ask permission to reach. Send the demo server made-up children only.

<div id="swagger-ui"></div>
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5.36.0/swagger-ui.css" integrity="sha384-z/AtSaOHGn5MJ5auAHXUypohywlIWLBZo1jeGmhdpVIbCnFhxAH92cwdV41oyINc" crossorigin="anonymous">
<script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5.36.0/swagger-ui-bundle.js" integrity="sha384-fUpRgC19yPykTqzdrUbdAg8NTiu16NEKf9U12J+IJWCg8/1mEI8f2+PEIz60Wyd+" crossorigin="anonymous"></script>
<script>
  window.addEventListener("load", function () {
    SwaggerUIBundle({
      url: "../openapi.json",
      dom_id: "#swagger-ui",
      deepLinking: true,
      defaultModelsExpandDepth: 0,
    });
  });
</script>
