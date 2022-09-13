### Chores
+ Update dependencies, removed anyhow, [98051e71a293b44b87ee02df8e3e4a409151e50b]

### Features
+ Store data in redis using redis hashes, [facbe98e4c9dd2d6c6ba0ea2f157584cc070029d]

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
