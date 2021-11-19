# Backup

该子命令用于创建和恢复备份

备份会将全部文章和评论以及当前配置项的值用msgpack编码并且brotli压缩成为单个`{date}.backup`文件.

恢复备份除了会将当前数据库**清空**, 然后再从备份中恢复. 同时将备份的配置项值输出到`{backup file name}.json`中.

## 参数
`-o/--output`: 备份文件输出路径. 默认为`{data_path}/backup`.
`-r/--recover`: 已有的`.backup`文件路径, 指定该选项等于**恢复**备份.

## Example
### 创建备份
```shell
maop backup
```

备份到`/home/me/maop-bk/xxx.backup`

```shell
maop backup -o /home/me/maop-bk
```

### 恢复备份
从`/home/me/maop-bk/xxx.backup`恢复:

```shell
maop backup -r /home/me/maop-bk/xxx.backup
```