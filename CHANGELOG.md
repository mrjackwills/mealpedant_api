# <a href='https://github.com/mrjackwills/mealpedant_api/releases/tag/v1.5.3'>v1.5.3</a>
### 2023-12-30

### Chores
+ Rust 1.75.0 linting, [d2a29ee9](https://github.com/mrjackwills/mealpedant_api/commit/d2a29ee94c838e513b14c8a9a46cce4f55b467f5)
+ dependencies updated, [58a9255f](https://github.com/mrjackwills/mealpedant_api/commit/58a9255f908c7a5a5fba7ff9ebd8e2aaa420f305)
+ Docker images alpine bump to 3.19, [0559ebbf](https://github.com/mrjackwills/mealpedant_api/commit/0559ebbf45bbe8ff2787841f4a1d24bf5d4da278)
+ .devcontainer updated, [68960fdb](https://github.com/mrjackwills/mealpedant_api/commit/68960fdb26195208f84325d96d540b34c87e6077)

### Features
+ run.sh v0.1.0, [08e47a96](https://github.com/mrjackwills/mealpedant_api/commit/08e47a9658b9e473957a28dea07e3f5453a872f4)
+ emailer & emalier template updated, [c1ad8e51](https://github.com/mrjackwills/mealpedant_api/commit/c1ad8e51496e0670cb2070676c93d99bcb4c0b37)

### Refactors
+ ttl rename, [d4236ab9](https://github.com/mrjackwills/mealpedant_api/commit/d4236ab964a07a29c5abc49f22479c1896815a54)

### Reverts
+ graceful shutdown reimplemented, [da5f37f1](https://github.com/mrjackwills/mealpedant_api/commit/da5f37f18322f9330150157b901ef196adcc6dcc)

# <a href='https://github.com/mrjackwills/mealpedant_api/releases/tag/v1.5.2'>v1.5.2</a>
### 2023-11-28

### Chores
+ axum upgraded to 0.7, [2a5413c7](https://github.com/mrjackwills/mealpedant_api/commit/2a5413c787cfb7d895afadebeca0d3865386ba50)

### Features
+ Use Arc for application state, [0aa755a9](https://github.com/mrjackwills/mealpedant_api/commit/0aa755a9329419829398384a6ca968f0862e46a3)

# <a href='https://github.com/mrjackwills/mealpedant_api/releases/tag/v1.5.1'>v1.5.1</a>
### 2023-11-17

### Chores
+ dependencies updated, [92154f9f](https://github.com/mrjackwills/mealpedant_api/commit/92154f9f254726f8cba1f871f545c1dfdda83f2b), [b937986f](https://github.com/mrjackwills/mealpedant_api/commit/b937986f3615fb5775f26191dc9273733e1e7899)
+ Rust 1.74.0 linting, [080166a8](https://github.com/mrjackwills/mealpedant_api/commit/080166a8a1843bcd4a42688fe5ccf8207681020a)
+ GitHub action updated, [b0ca4648](https://github.com/mrjackwills/mealpedant_api/commit/b0ca4648f71d3bb859fd3b42497cf9b50996b1ae)
+ .devcontainer updated, [075c8fa7](https://github.com/mrjackwills/mealpedant_api/commit/075c8fa758af13596949822da666222b4a75abad)
+ .gitattributes updated, [126ba671](https://github.com/mrjackwills/mealpedant_api/commit/126ba6718a601fe143b2d93693328c191eba2435)

### Fixes
+ banned domains postgres init, [14f54c03](https://github.com/mrjackwills/mealpedant_api/commit/14f54c03e4b33064918543d8b0f73de9bc0302d9)
+ Authentication> Authorization, [88327c13](https://github.com/mrjackwills/mealpedant_api/commit/88327c130e8ef81025ce12671bd67b3bb4dfb67f)
+ backup token authentication, [5a589c8d](https://github.com/mrjackwills/mealpedant_api/commit/5a589c8d815ff9db31d509e54d1cca34edcc0a55)

### Refactors
+ dead code removed, [57e371c3](https://github.com/mrjackwills/mealpedant_api/commit/57e371c345498c1ed0dde6f7ab5e3b4e391e42b6)

### Tests
+ banned email domain test update, [15336dfd](https://github.com/mrjackwills/mealpedant_api/commit/15336dfdf95d9c28595835045ed4d2f146a9eb13)

# <a href='https://github.com/mrjackwills/mealpedant_api/releases/tag/v1.5.0'>v1.5.0</a>
### 2023-10-06

### Chores
+ dependencies updated, [6ea1c028](https://github.com/mrjackwills/mealpedant_api/commit/6ea1c02857573fe96ac569e856ea2b1cf886e800)
+ postgres updated to 16, [49049e24](https://github.com/mrjackwills/mealpedant_api/commit/49049e24964123d38cfbca7da1655983d85738f4)
+ Rust 1.73.0 linting, [db734c7e](https://github.com/mrjackwills/mealpedant_api/commit/db734c7ebb020f66efd65ef07a2dcc792ab06351)

### Features
+ incoming json trimmed, [22645d7c](https://github.com/mrjackwills/mealpedant_api/commit/22645d7c368fac290187745dd835d8e1d1aac62e)

### Tests
+ Trimmed tests updated, [136d8316](https://github.com/mrjackwills/mealpedant_api/commit/136d83169a317527b0bb91c04b9e1db04591274e)

# <a href='https://github.com/mrjackwills/mealpedant_api/releases/tag/v1.4.3'>v1.4.3</a>
### 2023-08-25

### Chores
+ Rust 1.72.0 linting, [bdc0fa8d](https://github.com/mrjackwills/mealpedant_api/commit/bdc0fa8d49cdab26ac92ac736121233195576496)
+ dependencies updated, [816ea39f](https://github.com/mrjackwills/mealpedant_api/commit/816ea39f47c18abd15e7c4cd5c890799c06ed715)

# <a href='https://github.com/mrjackwills/mealpedant_api/releases/tag/v1.4.2'>v1.4.2</a>
### 2023-07-29

### Chores
+ dependencies updated, [7b41090c](https://github.com/mrjackwills/mealpedant_api/commit/7b41090cae2de6673e9bb557f258016200838d78)
+ create_release 0.3.0, [537a9fe1](https://github.com/mrjackwills/mealpedant_api/commit/537a9fe15b4b875f6396d516c665bf0e70e642e7)

### Features
+ define_routes!() macro, [8a3c37cd](https://github.com/mrjackwills/mealpedant_api/commit/8a3c37cdf2782bcbb97bae4bcc1ce6db5ca65935)

 # <a href='https://github.com/mrjackwills/mealpedant_api/releases/tag/v1.4.1'>v1.4.1</a>
### 2023-06-08


### Chores
+ dependencies updated, [919a988f](https://github.com/mrjackwills/mealpedant_api/commit/919a988f9dca02839a81db603dedbc03207bbae8), [5aff9bf0](https://github.com/mrjackwills/mealpedant_api/commit/5aff9bf085d3d633e4c27ded1ec529f88f8fc849)
+ Docker alpine bump to 3.18, [281da7a2](https://github.com/mrjackwills/mealpedant_api/commit/281da7a2fa9ba7c6d4408f2a9fcaa31de6e3030e)

### Features
+ `sleep!()` macro, [89baf679](https://github.com/mrjackwills/mealpedant_api/commit/89baf6797962f8b613d6e0cc2fa555fe794dddbd)
+ Cargo.toml lto thin, [0b2249eb](https://github.com/mrjackwills/mealpedant_api/commit/0b2249eb5fa3c035398262bdb88f2664cb367f6c)

### Fixes
+ get_prefix() removed, [3638c899](https://github.com/mrjackwills/mealpedant_api/commit/3638c8993cd88e64fcab44f0dc0bfa562d40be52)

### Reverts
+ .devcontainer sparse protocol now default, [5557a53b](https://github.com/mrjackwills/mealpedant_api/commit/5557a53bb74d0da6f1f80b5b0c6ffb8175be59d1)

# <a href='https://github.com/mrjackwills/mealpedant_api/releases/tag/v1.4.0'>v1.4.0</a>
### 2023-03-17

### Chores
+ devcontainer sparse protocol index, [ae6b36be](https://github.com/mrjackwills/mealpedant_api/commit/ae6b36be5e198684f236e606c4db730f79d89c61)
+ dependencies updated, [c8d550da](https://github.com/mrjackwills/mealpedant_api/commit/c8d550da59617679f74a99021f0b99255be7195b)
+ `base32` dev dependency removed, [e8fcb3f8](https://github.com/mrjackwills/mealpedant_api/commit/e8fcb3f8df8a81e61c662f0aa515f096c47ca80f)

### Features
+ use `totp-rs` for two factor auth, [7f35f1c0](https://github.com/mrjackwills/mealpedant_api/commit/7f35f1c03141d9f8bce1194b3d5f89fbe795eb66)
+ api.Dockerfile build from source, [4c2ef231](https://github.com/mrjackwills/mealpedant_api/commit/4c2ef2312999245f6f27e9391e5ab94f69ba50f0)

### Refactors
+ `SysInfo`, and use `tokio::fs`, [701bff73](https://github.com/mrjackwills/mealpedant_api/commit/701bff737d9ec656729d7ee62d672cb8f12ee45a), [ec3c9b29](https://github.com/mrjackwills/mealpedant_api/commit/ec3c9b291c49e0362a287ea1ab0f586e559d0e89)
+ use `unwrap_or`, [2aa6736c](https://github.com/mrjackwills/mealpedant_api/commit/2aa6736c371dd5fea7bae8481171aee91781a6cb)

### Tests
+ use `FLUSHDB` in tests, [d0071e38](https://github.com/mrjackwills/mealpedant_api/commit/d0071e3805809648ed0c804851d2447a9c4e1925)

# <a href='https://github.com/mrjackwills/mealpedant_api/releases/tag/v1.3.1'>v1.3.1</a>
### 2023-03-07

### Chores
+ Dockerfiles updated, [67601527](https://github.com/mrjackwills/mealpedant_api/commit/676015270958d969c4c50c83b486a040596a1392)
+ typos.toml add redis.conf, [ab466870](https://github.com/mrjackwills/mealpedant_api/commit/ab466870ed10a6acbd7928ab2f91ebdbee32a43a)

### Features
+ api dockerfile build from source, [b875b7a8](https://github.com/mrjackwills/mealpedant_api/commit/b875b7a8896f4d21ccd5c21105d5f7a560ed7bfc)
+ argon2, and associated methods, updated, [3a5837f6](https://github.com/mrjackwills/mealpedant_api/commit/3a5837f6cab19690ea514d172aa3b319dbe57a43)

### Fixes
+ create_release update, [3454e776](https://github.com/mrjackwills/mealpedant_api/commit/3454e776d890373624ce6fd1ce69d8901c0460df)
+ github action tag regex, [bf3ed841](https://github.com/mrjackwills/mealpedant_api/commit/bf3ed841505f72acfb21903be8c1350143e3c4b0)
+ serde_json downcast error fix, [2b360ade](https://github.com/mrjackwills/mealpedant_api/commit/2b360ade967a2639c3a291f26cf539eaec12aaa1)
+ api.Dockerfile missing gnupg dependency, [36c1aec4](https://github.com/mrjackwills/mealpedant_api/commit/36c1aec4fa5202d58b8c0bf3c13ce16a1cf93d4c)

### Refactors
+ postgreSQL queries use `USING(x)` where appropriate, [a6b76a16](https://github.com/mrjackwills/mealpedant_api/commit/a6b76a16f984a6ccef909d1624424161766e201e)

# <a href='https://github.com/mrjackwills/mealpedant_api/releases/tag/v1.3.0'>v1.3.0</a>
### 2023-02-11

### Chores
+ multiple typos, [1354e516](https://github.com/mrjackwills/mealpedant_api/commit/1354e516a485c0739bd705d4075678d81fd41556)
+ dev container updated, [4fd27b2e](https://github.com/mrjackwills/mealpedant_api/commit/4fd27b2e1c97e831cea38903afd0da1f64c6748c), [dccce72c](https://github.com/mrjackwills/mealpedant_api/commit/dccce72c9dd48e2c4685b5adcc129d45ec6792da)
+ dependencies updated, openssl removed, [7e256374](https://github.com/mrjackwills/mealpedant_api/commit/7e256374cdb6089c2c9fdfb97a58a7c83fbc4ba0)

### Features
+ use age for encryption, [35fca144](https://github.com/mrjackwills/mealpedant_api/commit/35fca1446b3d527a77b7d1365488d26c16c91c1f)
+ dev docker mount db's in ram, [dc1402ef](https://github.com/mrjackwills/mealpedant_api/commit/dc1402ef424f8cd2163c9b6dfb6d94315a42dd42)

### Refactors
+ Handle uuid parsing errors manually, [894f7231](https://github.com/mrjackwills/mealpedant_api/commit/894f723146a66b84464a02b4e63308259157764f)

# <a href='https://github.com/mrjackwills/mealpedant_api/releases/tag/v1.2.3'>v1.2.3</a>
### 2023-01-23

### Chores
+ dependencies updated, [9c765c1c](https://github.com/mrjackwills/mealpedant_api/commit/9c765c1cd26fcf32692024edecf72738d08e0610)

### Features
+ AppEnv use enum instead of bools, [72cff0cb](https://github.com/mrjackwills/mealpedant_api/commit/72cff0cb13b2df1a366f2b74af02ccf7aee88bc2)

### Fixes
+ add timeout to hibp request, [970bb193](https://github.com/mrjackwills/mealpedant_api/commit/970bb1933f357f673317b9fd5037e90f460ef16b)
+ ratelimit ttl isize, [3564d980](https://github.com/mrjackwills/mealpedant_api/commit/3564d980dba1acc6f348de1e9a8951e3fe3c43d0)

### Refactors
+ store log level in app_env directly, [f421cdc1](https://github.com/mrjackwills/mealpedant_api/commit/f421cdc1c2266c6f7fa1bb81eecc0f23826b905c)
+ authentication, [02b15aac](https://github.com/mrjackwills/mealpedant_api/commit/02b15aac332790e7f459fe35f7fddb322dc24d9d)

### Tests
+ two_fa_always required method, [794ecf9a](https://github.com/mrjackwills/mealpedant_api/commit/794ecf9adfd8f05694b7b9454dcba37f48d7d6d4)

# <a href='https://github.com/mrjackwills/mealpedant_api/releases/tag/v1.2.2'>v1.2.2</a>
### 2022-12-17

### Chores
+ dependencies updated, [1d8ca5fc](https://github.com/mrjackwills/mealpedant_api/commit/1d8ca5fc20f2b93accee7322637671224921fecc)

### Features
+ remove body size on photo post, [84e7062e](https://github.com/mrjackwills/mealpedant_api/commit/84e7062e30c4f33a6cf70b0dd3bb68b0e86f1722)

### Fixes
+ test_image.jpg increase size to > 3mb, [bb798523](https://github.com/mrjackwills/mealpedant_api/commit/bb7985239851c990e04e04d63543c4add00f15ab)
+ docker container(s) use ubuntu, [da07074a](https://github.com/mrjackwills/mealpedant_api/commit/da07074a80aa2c5872315654220727efc9cec2c7)

# <a href='https://github.com/mrjackwills/mealpedant_api/releases/tag/v1.2.1'>v1.2.1</a>
### 2022-12-15

### Chores
+ linting with Rust 1.66, [7d8a5542](https://github.com/mrjackwills/mealpedant_api/commit/7d8a554281593a02bfe219bd861ba4095c6f224e)
+ docker alpine bump, [5a754b86](https://github.com/mrjackwills/mealpedant_api/commit/5a754b86b467ddaeac54691cdacb408d778e26f8)
+ dependencies updated, [8c36a2d4](https://github.com/mrjackwills/mealpedant_api/commit/8c36a2d4127517dde4fb6b4c3a0c0b88b1c2bd72), [39522f36](https://github.com/mrjackwills/mealpedant_api/commit/39522f3667f3e807c0a56c39a6d05b297c2fcd56)

### Features
+ use tuple struct for Argonhash, [e6d161b8](https://github.com/mrjackwills/mealpedant_api/commit/e6d161b830336731520fa83bb18e801ad7cb925e)
+ github action workflow use rust cache, [c2d33297](https://github.com/mrjackwills/mealpedant_api/commit/c2d332970a3e66ad660dbf931311480b96b53cb1)

### Fixes
+ session ttl usize try_from, [d4916d15](https://github.com/mrjackwills/mealpedant_api/commit/d4916d15aaef4d2637b8197e355065369e12f866)
+ redundant cargo-watch install in devcontainer.json removed, [02880955](https://github.com/mrjackwills/mealpedant_api/commit/028809557fde88a9b540347d8811d786fa1dac86)

### Refactors
+ remove Deserialize from ModelMeal, [8cb580c9](https://github.com/mrjackwills/mealpedant_api/commit/8cb580c9b2f7928f65ec0761dd2e55229d1e25b2)
+ use cookiejar as param, rather than extracting from parts, [b177b9b2](https://github.com/mrjackwills/mealpedant_api/commit/b177b9b2599facbb4d11e558a2c83726184ada8a)


# <a href='https://github.com/mrjackwills/mealpedant_api/releases/tag/v1.2.0'>v1.2.0</a>
### 2022-11-26

### Chores
+ aggressive linting with rust 1.65.0, [94f84080](https://github.com/mrjackwills/mealpedant_api/commit/94f840801138eb8d7b8a2f35a2e4241fed4c7d4f)
+ dependencies updated, [5a8d4a53](https://github.com/mrjackwills/mealpedant_api/commit/5a8d4a53e9fdc02c6ffbdae09d84a4bca10f722e)
+ create_release v0.1.2, [c7718421](https://github.com/mrjackwills/mealpedant_api/commit/c7718421106861779323c4b10c4d6302be508b7f)
+ dotenv > dotenvy, uuid upgraded, [9e74681c](https://github.com/mrjackwills/mealpedant_api/commit/9e74681c9baf2fb7801b3c746944e527800a7f98)

### Docs
+ readme, [4002fd73](https://github.com/mrjackwills/mealpedant_api/commit/4002fd73c33d0f95b7a4015e72af4e729cd3757d)

### Features
+ replace lazy_static with once_cell, [26305f55](https://github.com/mrjackwills/mealpedant_api/commit/26305f55a85eae7b6adcfa72ae854545e680d459)
+ update axum to 0.6, [40d5d0c4](https://github.com/mrjackwills/mealpedant_api/commit/40d5d0c4501e9437381a0df5422d236102d3555a)
+ upgrade postgres to 15, [d7ffd017](https://github.com/mrjackwills/mealpedant_api/commit/d7ffd017a012c649771a7471b664512a371b3060)
+ use dtolnay/rust-toolchain in github workflow, [8f1182b6](https://github.com/mrjackwills/mealpedant_api/commit/8f1182b60233499249f8a74f0cd8fb559219f6e1)

### Fixes
+ track Cargo.lock, [be606910](https://github.com/mrjackwills/mealpedant_api/commit/be6069108e8df99eebda409658350871772e8bed)
+ rate limiter fix, [b250b12e](https://github.com/mrjackwills/mealpedant_api/commit/b250b12ea101283a42318a1c7805eb71ed6eb63e)
+ photo name filename parse fix, [6b2429aa](https://github.com/mrjackwills/mealpedant_api/commit/6b2429aa7fc2225e509c384db5f2aac3dd8d9f0d)

### Refactors
+ map_or_else to map_or, [b0b3755e](https://github.com/mrjackwills/mealpedant_api/commit/b0b3755ef15f84224615df4ce73d4ede777dd3d6)
+ get_addr into own function, [925641da](https://github.com/mrjackwills/mealpedant_api/commit/925641daa1d3082aae0dc6dcd1e455de5933f309)
+ ratelimiter fix, [cd7b7554](https://github.com/mrjackwills/mealpedant_api/commit/cd7b7554a8c99cb593030134552f6ee11aff9fb3)

# <a href='https://github.com/mrjackwills/mealpedant_api/releases/tag/v1.1.0'>v1.1.0</a>
### 2022-09-13

### Chores
+ Update dependencies, removed anyhow, [98051e71](https://github.com/mrjackwills/mealpedant_api/commit/98051e71a293b44b87ee02df8e3e4a409151e50b),

### Features
+ Store data in redis using redis hashes, [facbe98e](https://github.com/mrjackwills/mealpedant_api/commit/facbe98e4c9dd2d6c6ba0ea2f157584cc070029d),

# <a href='https://github.com/mrjackwills/mealpedant_api/releases/tag/v1.0.3'>v1.0.3</a>
### 2022-08-12

### Chores
+ aggressive linting, [43214fea](https://github.com/mrjackwills/mealpedant_api/commit/43214fea2160dc63689b7203efc5dacd37a25705),
+ dependency removed, [cd877118](https://github.com/mrjackwills/mealpedant_api/commit/cd8771186b60bdb1eb35ace6430cd5772707230a),
+ dev dockerfile updated, [b0f8c60a](https://github.com/mrjackwills/mealpedant_api/commit/b0f8c60ae04bf7477f45b08ffd92bc9a0cd3f6e0),
+ dependencies updated, [ebc45a15](https://github.com/mrjackwills/mealpedant_api/commit/ebc45a15786ff98c01b339b55b0b0d9b064216a3),

### Docs
+ Readme.md tweak, [12b7ecdc](https://github.com/mrjackwills/mealpedant_api/commit/12b7ecdc81158caeb7f5a7b71001430c99b03d5c),

### Features
+ Logs written to disk as JSON, parsed & sent as JSON to frontend, [6357a2d3](https://github.com/mrjackwills/mealpedant_api/commit/6357a2d34ebfb3b36e59be5c6d8e861e5ae6b472),
+ Switch api.Dockerfile from Alpine to Debian bullseye, [c0ed43b2](https://github.com/mrjackwills/mealpedant_api/commit/c0ed43b2a706d291a2b299f01f51f3f52732ac75),



# <a href='https://github.com/mrjackwills/mealpedant_api/releases/tag/v1.0.2'>v1.0.2</a>
### 2022-08-03

### Chores
+ dependencies updates, [b44222db](https://github.com/mrjackwills/mealpedant_api/commit/b44222dbc704262dfdfe6883e40057b14ad6b162),
+ dev docker context changed, [53287f6c](https://github.com/mrjackwills/mealpedant_api/commit/53287f6cb1d6c3192fdba2d8c0a58947e657b0aa),, [efd0a745](https://github.com/mrjackwills/mealpedant_api/commit/efd0a7452b008427c2dfdfdfb9ad1a87d90ec29c),
+ linting rule - clippy::unwrap_used, [587e7a5a](https://github.com/mrjackwills/mealpedant_api/commit/587e7a5a839aa90b0b0fcfa18e552c253885d47d),, [a40d92da](https://github.com/mrjackwills/mealpedant_api/commit/a40d92da12150b5cfae261f1a3fe80f0ec08cf8c),
+ nursery linting, [a450e694](https://github.com/mrjackwills/mealpedant_api/commit/a450e6949870fe5c5be41cff7611a86432043428),
+ pedantic linting, [a77f9bb3](https://github.com/mrjackwills/mealpedant_api/commit/a77f9bb31dab4a57226710da20ce799dfd2f1a6c),

### Docs
+ readme typos, [fe7784ac](https://github.com/mrjackwills/mealpedant_api/commit/fe7784aca5ea3e61aeeb2ae68cc96a524e255e70),
+ Licence added, [d3a59f7c](https://github.com/mrjackwills/mealpedant_api/commit/d3a59f7c02806b3c9e9de6f039c961ce34a86d94),

### Features
+ create_release.sh build for production, [31f0962b](https://github.com/mrjackwills/mealpedant_api/commit/31f0962b10e5c14ef68caef7a995c387823706cc),

### Fixes
+ authenticate split into signing & password+token, [e392ff80](https://github.com/mrjackwills/mealpedant_api/commit/e392ff80d44931ae72ca25af04375800df5057da),


# <a href='https://github.com/mrjackwills/mealpedant_api/releases/tag/v1.0.1'>v1.0.1</a>
### 2022-07-20

### Fixes
+ Order list of backup files by date, [4ff7afdd](https://github.com/mrjackwills/mealpedant_api/commit/4ff7afdd9f880524a1c41f7f1d6577949c47c3ce),
+ use env to build redis.conf, [c2a6d286](https://github.com/mrjackwills/mealpedant_api/commit/c2a6d286bbf033117e4d7352f8ae9ed5b4faffab),
+ create_release auto update readme.md hyperlink, [8d28a0c4](https://github.com/mrjackwills/mealpedant_api/commit/8d28a0c419efa01f1e7c4d456eb571cd2e94ddea),
+ docker-compose, bind to exact .env file, [31c22280](https://github.com/mrjackwills/mealpedant_api/commit/31c22280d19cb78baba3e15a6cb614e61b3f3c92),
+ docker sql.init updated, [20695638](https://github.com/mrjackwills/mealpedant_api/commit/20695638141b371a41e936ef43bf168417dc6d98),

# <a href='https://github.com/mrjackwills/mealpedant_api/releases/tag/v1.0.0'>v1.0.0</a>
### 2022-07-19

### Features
+ init release
