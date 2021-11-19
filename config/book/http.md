# http配置

`[http]`段为http服务器相关配置.

| 字段 | 类型 | 描述 | 默认值 |
| --- | --- | --- | --- |
| `bind` | String | 绑定的地址 | 127.0.0.1 |
| `port` | Number | 监听http端口 | 7474 |
| `type` | `http` &#124; `uds` | 通信类型 | `http` |
| `session_expiry` | Time | session过期时间 | `7day` |
| `overdue_check_interval` | Time | 定时清理过期session间隔 | `5h` |
| `cors` | Array[String] | 允许的origin | `[]` |

## Unix Domain Socket 
将`type`字段设置为`uds`, 然后`bind`设置为文件地址即可.

```toml
[http]
type = "uds"
bind = "/home/me/maop.run"
```

## Cors
`*`表示接受任意origin

```toml
[http]
cors = ["*"]
```
