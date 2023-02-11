<p align="center">
 <img src='./.github/logo.svg' width='125px'/>
 <br>
 <em>A meticulous daily log of ingestion</em>
 <h1 align="center">
 <a href='https://api.mealpedant.com/v1/incognito/online' target='_blank' rel='noopener noreferrer'>api.mealpedant.com</a>
  </h1>
</p>

<p align="center"><em>Since May 9th 2015, two transatlantic friends have pedantically exchanged information on every evening meal that they have consumed. This is a comprehensive chronicling of that pedantry.</em></p>
<hr>

<p align="center">
	Built in <a href='https://www.rust-lang.org/' target='_blank' rel='noopener noreferrer'>Rust</a>
	for <a href='https://www.docker.com/' target='_blank' rel='noopener noreferrer'>Docker</a>,
	using <a href='https://www.postgresql.org/' target='_blank' rel='noopener noreferrer'>PostgreSQL</a>
	& <a href='https://www.redis.io/' target='_blank' rel='noopener noreferrer'>Redis</a> 
	<br>
	<sub> See typescript branch for original typescript version</sub>
</p>

<hr>

<p align="center">
	This is the backend for <a href='https://www.github.com/mrjackwills/mealpedant_vue' target='_blank' rel='noopener noreferrer'>mealpedant_vue</a> front-end, and is an on-going work in progress.
	<br>
	The backend is a CRUD api application with the following features;
	<ul>
		<li><a href="https://github.com/tokio-rs/axum" target='_blank' rel='noopener noreferrer'>axum</a> based with <a href='https://www.tokio.rs' target='_blank' rel='noopener noreferrer'>tokio</a> async mutlithreading</a>
		<li>Password hashing with argon2</li>
		<li>Weak password resolution & rejection, powered by <a href='https://haveibeenpwned.com/' target='_blank' rel='noopener noreferrer'>hibp</a></li>
		<li>Time based Two-Factor Authentication</li>
		<li>Two-Factor Authentication backup codes</li>
		<li>User sessions using private, encrypted, cookies, with a redis backend</li>
		<li>redis backed login, and/or ip and/or user_id rate limiting</li>
		<li>Automated email templating & sending, using <a href='https://mjml.io/' target='_blank' rel='noopener noreferrer'>mjml</a></li>
		<li>User & Admin user accounts</li>
		<li>Restricted User area</li>
		<li>Restricted Admin user area</li>
		<li>strict CORS settings</li>
		<li>Multi-part uploads - for images of meals</li>
		<li>Image conversion, resizing & watermarking</li>
		<li>Customised incoming serde serialization & extraction</li>
		<li>Error tracing</li>
		<li>Redis based cache</li>
		<li>Postgres main data store</li>
		<li>Scheduled automated database backup & encryption</li>
		<li>(attempted complete) test coverage</li>
		<li>Automated github build step</li>
		<li>Fully Dockerized production environment</li>
		<li>Development remote container (using <a href="https://code.visualstudio.com/docs/remote/containers" target='_blank' rel='noopener noreferrer'>vscode</a>)</li>
	</ul>
<p>

### Todo
+ improve backup creation, currently consumes large amount of memory, maybe separate into own container?
---

## Download

See <a href="https://github.com/mrjackwills/mealpedant_api/releases" target='_blank' rel='noopener noreferrer'>releases</a>

download (x86_64 one liner)

```bash
wget https://www.github.com/mrjackwills/mealpedant_api/releases/latest/download/mealpedant_linux_x86_64.tar.gz &&
tar xzvf mealpedant_linux_x86_64.tar.gz mealpedant
```

## Run

Operate docker compose containers via

```bash
./run.sh
```

## Build

```bash
cargo build --release
```
<strike>
Build using <a href='https://github.com/cross-rs/cross' target='_blank' rel='noopener noreferrer'>cross</a>, for x86_64 linux musl targets, in order to run in an Alpine based container

```bash
cross build --target x86_64-unknown-linux-musl --release
```
</strike>

## Tests

Requires postgres & redis to both be operational and seeded with valid data - <a href="https://github.com/mrjackwills/mealpedant_api/blob/main/src/database/postgres/init.sql" target='_blank' rel='noopener noreferrer'>init.sql</a> contains database structure, but for food privacy reasons, the full meal data cannot be provided

```bash
# Watch
cargo watch -q -c -w src/ -x 'test -- --test-threads=1 --nocapture'

# Run all 
cargo test -- --test-threads=1 --nocapture

# Test coverage, requires cargo-llvm-cov to be installed globally
# then: rustup component add llvm-tools-preview --toolchain 1.61.0-x86_64-unknown-linux-gnu
cargo llvm-cov -- --test-threads=1
```
