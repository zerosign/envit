# serde-env

serde (serializer &amp; deserializer) for environment, supports for tree &amp; array

## Supported formats (TODO)

- Dictionary like data
- Array like data
- Optional environment
- enum data

  ```bash
  CONFIG__DATABASE__NAME=name
  CONFIG__DATABASE__USERNAME=username
  CONFIG__DATABASE__URL=mysql://host:port
  CONFIG__DATABASE__CREDENTIAL__TYPE=password
  CONFIG__DATABASE__CREDENTIAL__PASSWORD=some_password
  CONFIG__DATABASE__CONNECTION__POOL=10
  CONFIG__DATABASE__CONNECTION__TIMEOUT=10
  CONFIG__DATABASE__CONNECTION__RETRY=10,20,30
  CONFIG__APPLICATION__ENV=development
  CONFIG__APPLICATION__LOGGER__LEVEL=info

  # config {
  #   database: {
  #     name: name,
  #     username: username,
  #     url: mysql://host:port,
  #   },
  #   credential: {
  #     type: password,
  #     password: some_password,
  #   },
  #   connection : {
  #     pool: 10,
  #     timeout: 10,
  #     retry: [10, 20, 30],
  #   },
  #   application: {
  #     env: Development,
  #   },
  #   logger: {
  #     level : Info,
  #   }
  # }
  ```
