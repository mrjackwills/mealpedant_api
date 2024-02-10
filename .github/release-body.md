### 2024-02-10

### Chores
+ GitHub action improved, [60cb57b2948762dd0b75e0bc1e67349f11c4f8d2]
+ run.sh updated, [c490a4b745178ec975d397234245a58a6f3f13b0]
+ .devcontainer updated, [8b3f26f0f8daaa661c0339881a266f8d968169b6]
+ create_release.sh v0.5.3, [dbba4292b6f85861b35f1ae20e64799d24933632]
+ dependencies updated, [a277bdf4c3a14e411a07e0de77f95924267c6f84]

### Features
+ replace redis with fred, [da96cad59ee6befb914c32acd4e6e5e406bcc048]

### Fixes
+ change Arc<Mutex<redis>> to ConnectionManager (now redundant), [d7ad94a3b53ff1d52915fa2756b70402bf35c856]
+ docker-compose api memory limit increased, [da787db65d64841c3afe6bab1455a7bb38222004]
+ argon cfg debug, [4ad24be544e28677623e3bc436d32f1297116948]

### Refactors
+ internal!() macro in api_error, [8cc4fc88fe6280d22f9c429bb444ea7b5408e45a]
+ Dockerfiles & setup simplified, [3875b6b1f7c59473992743805923298bb854f17b]
+ use into_iter(), [06bf23ea7d2cd1c84f81af06e5c2d287ec801f55]

### Tests
+ login with backup code refactored, missing tests added, [407097d0fd87d59b75974ef4728f756a3a2c60f9]
+ test: uuid test added, [e2065717f4983a05de51aaba4eba0f049b456b70]

see <a href='https://github.com/mrjackwills/mealpedant_api/blob/main/CHANGELOG.md'>CHANGELOG.md</a> for more details
