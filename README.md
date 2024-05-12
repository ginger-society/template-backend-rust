Fully working CRUD REST API example using 
- Rust (stable)
- Rocket.rs
- mongodb
- okapi


## 🚀 Features
- Establish MongoDB connection using rocket Adhoc fairing.
- Custom error handlings with rocket Responder and okapi OpenApiGenerator.
- CORS fairing and Counter fairing to demonstrate how fairing works.
- Example model Customer to demonstrate how Rust structs interact with MongoDB.
- Request guard using ApiKey.
- REST API endpoints with simple CRUD using Customer model.
- Implement Open API documentation using okapi.
- Test codes to test API endpoints.


## 🔧 Building and Testing

### debug mode
> cargo run

### release mode
> cargo build --release && cargo run --release


### unit testing
> cargo test

<br/>

ℹ️ _You should create your own `.env` file including `MONGO_URI`, `MONGO_DB_NAME`, and `API_KEY` to run it._

diesel print-schema > src/schema_generated.rs
diesel_ext --model > src/models_generated.rs  