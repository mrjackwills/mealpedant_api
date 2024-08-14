### 2024-08-14

### Chores
+ .devcontainer docker-in-docker, [b9d00496cbf5dd85c17bf09aafb5b3d5a30d7ac8]
+ dependencies updated, [3dd66ac7f42e6eea83352d53f92976aa70da7a22]

### Features
+ switch from `/dev/shm` to `/ramdrive`, [24e47958c45cb8fc8516bcfd60c8f893e59718cf]

### Fixes
+ api healthcheck more robust and *correct*, [748c6a4fe235db91c172bd27d067fe38055a4ae7]
+ run.sh directories location, [9f5a3810b1775b55bf658c20431c56d447246264]
+ increase api Docker memory limits, [3fc61417663574ef29c0987b8b128cb21dc6b726]

### Refactors
+ replace OnceCell with std::sync::LazyLock, [40c41d9f9fc7279ffe928aa203cae4b59ce9d12c]

see <a href='https://github.com/mrjackwills/mealpedant_api/blob/main/CHANGELOG.md'>CHANGELOG.md</a> for more details
