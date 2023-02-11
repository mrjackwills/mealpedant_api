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
