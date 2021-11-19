
## 单位

### 时间
时间解析使用[parse_duration](https://docs.rs/parse_duration/).
具体语法可以[看这里](https://docs.rs/parse_duration/).

一般格式为数字+单位, 例如`30s`, `24h`, `1 minutes 39 seconds`等.

### 数据大小
~~懂的都懂~~

`1 B`, `1 KB`, `1 MIB`...
> KB for 1000 bytes, KiB for 1024 bytes

最高支持PIB级大小.
