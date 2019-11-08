# rocket-postgresql-ssl
The code for Rocket managed state / request guard / Postgresql SSL databases.
As Rocket #[database] implementation does not (yet) handle SSL, we must use managed state and request guard.

Issue in Rocket repo:
https://github.com/SergioBenitez/Rocket/issues/1115

"Old" database documentation used for the code:
https://rocket.rs/v0.3/guide/state/#databases

The blog which greatly helped me:
https://matthewkmayer.github.io/blag/public/post/postgres-tls/


In your main, add:
```
mod database;
```

and before Rocket launch:
```
rocket::ignite()
  ...
  .manage(init_pg_pool())
  .launch();
```
