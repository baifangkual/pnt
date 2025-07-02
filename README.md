# pnt
A simple TUI password note application

## 暂记

将原使用的bfk-passwd-note用rs-tui重写

### 待完成

* 先总体实现，再补充加密等细节
* 主密码校验和主密码解密密文不应使用一个hash，
应当主密码的校验hash和主密码解密密文hash使用两个不同的hash，
这样即使拿到了主密码校验hash,也无法通过主密码校验hash来获取主密码解密密文hash
* 由于sqlite数据库为静态文件，所以无法抵挡暴力破解，需要对数据库文件进行尽可能的加密
* 是否应确定不同的编译的bin的校验不一？
* db不应当存储主密码，而是应当存储对主密码单向加密的结果
* pnt不应该防止彩虹表攻击，因为攻击者已经拿到了静态的数据库文件，防止彩虹表除了降低效率无意义，
同理，也不必设定更新主密码的单向向量，以及主密码验证失败时的延迟...
* 内存中显示存储的密文后应立即清楚该内存段覆写
* 后续应扩展：数据库中存储至少v1,v2,v3三个字段，
三个字段应当为identity,passwd,wrap_pwd(随机向量-加密identity和passwd的密码，每个条目一个)，
并且每个条目的identity,passwd,wrap_pwd存放到v1,v2,v3的位置应当不同，
因为使用自增id且每个条目id不变，遂可以通过自增id来确定v1,v2,v3的位置
* 后续应扩展：上浮一层抽象层，可选配置加密update,crate时间，透露更少信息
* 后续应扩展：提供导出备份功能
* 后续应扩展：pnt tui 程序运行时，若使用运行时需要密码，则每次查看密文时都验证界面是否运行了很久，
若超过一定阈值，则要求输入主密码
* kv表无需添加向量，因为kv表其是针对pnt的，pnt无任何秘密可言


## License

Copyright (c) baifangkual

[MIT] OR [Apache-2.0]

[MIT]: ./LICENSE-MIT
[Apache-2.0]: ./LICENSE-APACHE