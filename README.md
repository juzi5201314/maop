# maop

[![Check](https://github.com/juzi5201314/maop/actions/workflows/ci.yaml/badge.svg)](https://github.com/juzi5201314/maop/actions/workflows/ci.yaml)

一个旨在简洁但功能强大的个人博客程序

| [Book](https://maop-book.soeur.dev/) | [Example](https://maop.soeur.dev) |

## 设计宗旨
- 易于使用。启动一个最小实例应该非常简单，不应该需要过多配置才能使用。
- 功能强大。在易于使用的前提下，可配置的功能尽可能多。
- 性能合格。功能尽可能多，但不应该过多影响性能，尽量(
- 目标是个人博客。没有多用户等等既影响开发难度，也影响使用难度的东西。
- 开发自由。
  - 这是一个使用宽松许可证的个人项目，非常欢迎fork和贡献。
  - 这是一个binary而不是library，所以不需要考虑兼容-
    可以尽情使用nightly版本编译器和nightly feature，激进的msrv
  - 这是一个个人项目，所以unsafe是可以的，没有unsafe警察。
    只要我们认为它是safe的。如果出现了问题，改正就好了。

## 特点
基于上面的设计理念
- maop拥有非常细致的配置项，比如数据库连接,分模块日志等级过滤等等，
  但都有着合理的默认值，启动时最多只需要在第一次启动时输入密码(甚至密码也可以暂时省略)
  ，什么都不需要配置也可以良好运行。
- 良好的性能和易于维护，使用rust开发，先天比php的typecho有优势，而且
  rust代码更容易维护和开发。
- 更多的bug，因为这是一个菜鸟开发的，而好处在于可以激励你为开源项目做贡献(~~指帮我修bug~~

## 目标用户
- 至少得会在windows/linux下安装软件(rust)并编译本程序。
(虽然提供二进制分发，但启用额外feature需要手动编译程序)

- 有小学级别的自学能力，因为配置文件是toml格式的(至少得学懂toml格式)

- 有截图键或者ctrl和cv键。我非常乐意解答任何问题，但至少要有错误信息。

## 性能
~~最大的妥协，比typecho性能好~~
还在开发中，没有进行针对性能的优化。但在开发过程中尽可能的考虑性能问题。

## 为什么没有xxx
别骂了别骂了，在写了在写了。

有好玩/有用的功能欢迎讨论。
