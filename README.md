抖音的开放平台的第三方SDK

# 说明

这是一个，个人（被动）维护的，的抖音开放平台的RustSDK项目
仅限使用到的API进行封装，不包含其他API，后续可能会逐步增加其他API
如果您需要对这个SDK进行扩展，包括项目结构调整，提供功能说明，或添加测试用例，提交PR，或发送邮件。稍后我会对项目进行更新。

对应的 抖音官方文档为[https://developer.open-douyin.com/docs/resource/zh-CN/interaction/develop/server/server-api-introduction]()

我刚看到抖音的服务端分为很多版块，这里只是直播小玩法的服务端文档，由于对文档不熟悉，不知道如何抽取公共功能。针对版本号~0.1，仅对其扩展不做破坏性变更

这只是一个练手项目，对于rust我还有很多困惑，也不知道如何精简项目，欢迎对Rust进行学习和交流。


# 关于依赖

这是我发布的第一个Rust项目，我不确定需不需要在您的cargo中添加依赖,所以我把依赖列表放在这里
```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.11.26", features = ["json"] }
rand = "0.8.5"
rsa={version="0.9", features = ["sha2"]}
base64="0.22"
md5 = "0.7"
```


# Example

```rust
    let app_private_key = include_str!("private_key.pem");
    let config = DouyinConfig {
        appid: "appid",
        secret: "secret",
        app_private_key: app_private_key,
        ..Default::default()
    };
    let mut sdk = SDK::new(config);

    // 直播小玩法->开发->服务端->接口调用凭证->getAccessToken->获取access_token
    let token = sdk.get_access_token().await;
    // 直播小玩法->开发->服务端->直播能力->数据开放->启动任务
    let start_res = sdk.task::<LiveOpenReqDataStart>("start","roomid","appid","msg_type").await;
    // 直播小玩法->开发->服务端->直播能力->数据开放->停止任务
    let stop_res = sdk.task::<LiveOpenReqDataStop>("stop","roomid","appid","msg_type").await;
    // 直播小玩法->开发->服务端->直播能力->数据开放->查询任务状态
    let status_res = sdk.task::<LiveOpenReqDataStatus>("status","roomid","appid","msg_type").await;
    // 直播小玩法->开发->服务端->直播能力->直播信息
    let info = sdk.info("exe启动时携带的token").await;
    // 直播小玩法->开发->服务端->直播能力->数据开放->验证签名
    let str = sdk.verify_sign(sign_map,&body,&app_secret);

    let random_str = make_random_string();
    let ts = get_now_timestamp(false);
````