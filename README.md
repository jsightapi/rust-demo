# How to run rust demo server with JSight

1. Go to `./docker` folder: `cd docker`.
2. Run docker compose: `docker compose up`.
3. Go inside the running docker: `docker exec -it rust-builder bash`.
4. Inside the docker `\opt\app` folder start server by the command `cargo run`.
5. Send requests to `localhost:8000` according to API spec in `./app/src/my-api-spec.jst`.  
   Examples:  
   Good request: `curl --location 'http://localhost:8000/users' --data '{"id": 1, "login": "l"}'`.  
   Bad request: `curl --location 'http://localhost:8000/users' --data '{"id": 1}'`.


**IMPORTANT!** Do not forget to remove `jsight::clear_cache()` line in production mode.
Clearing cache reduces performance dramatically.