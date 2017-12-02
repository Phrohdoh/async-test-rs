# `async-test`

This code showcases responding to (and making) HTTP GET requests in an asynchronous fashion.

### Running

```
cargo run
```

### Usage

Once the server is running run the following command from another shell:

```
curl http://127.0.0.1:3000/first
```

Notice that the server responds immediatly and does its work in the background so the client can carry on.