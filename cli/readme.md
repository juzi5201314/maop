# Cli

## 子命令
* [backup](./book/backup.md)
* [password](./book/password.md)

## 参数
`-c/--config`: 指定配置文件路径, 可指定多个. example: `maop -c xx.toml -c yy.json`

`-e/--env`: 指定`.env`文件路径.

`--no_password`: 无密码启动.

## 注意
无密码启动并非不需要密码就能登录, 恰恰相反, 是**任何人都无法登录**, 没有密码就永远不会有正确密码.
