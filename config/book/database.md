# 数据库配置

`[database]`段为数据库相关配置.

| 字段 | 类型 | 描述 | 默认值 |
| --- | --- | --- | --- |
| `timeout` | Time | 连接超时 | `5s` |
| `max_conn` | Number | 连接池最大连接数 | 50 |
| `min_conn` | Number | 连接池最小连接数 | 1 |
| `max_lifetime` | Time | 连接最大生命周期 | `12h` |
| `idle_timeout` | Time | 连接空闲超时 | `1h` |
| `warn_time` | Time | 操作超时警告 | `3s` |
| `shared_cache` | Bool | sqlite配置项 | false |
| `statement_cache_capacity` | Number | sqlite配置项 | 100 |
| `page_size` | Number | sqlite配置项 | 4096 |

* `max_lifetime`: 单条连接保持太久可能会导致数据库方面的资源泄露.
* `min_conn`: 在连接池中保持一定连接可以加快冷访问速度. 
突然有少量用户访问时不需要打开新连接.
* `warn_time`: 单个操作超时会发出warning日志.
