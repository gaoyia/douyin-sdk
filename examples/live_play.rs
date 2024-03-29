extern crate douyin_sdk; // 导入lib.rs中的库

use douyin_sdk::{get_now_timestamp, make_random_string, map_2_str, DouyinConfig, LiveOpenReqDataStart, LiveOpenReqDataStatus, LiveOpenReqDataStop, SDK}; // 使用lib.rs中的函数

#[tokio::main]
async fn main ()  {
    // 它将在编译时写进二进制可执行文件中。如果需要在运行时更新请自行修改。
    let app_private_key = include_str!("private_key.pem"); // 请自行修改私钥，不要使用示例中的
    let config = DouyinConfig {
        appid: "appid",
        secret: "secret",
        app_private_key: app_private_key,
        ..Default::default()
    };
    let mut sdk = SDK::new(config);
    // 直播小玩法->开发->服务端->接口调用凭证->getAccessToken->获取access_token
    let access_token = sdk.get_access_token().await;
    // 直播小玩法->开发->服务端->直播能力->数据开放->启动任务
    let start_res = sdk.task::<LiveOpenReqDataStart>("start","roomid","appid","msg_type").await;
    // 直播小玩法->开发->服务端->直播能力->数据开放->停止任务
    let stop_res = sdk.task::<LiveOpenReqDataStop>("stop","roomid","appid","msg_type").await;
    // 直播小玩法->开发->服务端->直播能力->数据开放->查询任务状态
    let status_res = sdk.task::<LiveOpenReqDataStatus>("status","roomid","appid","msg_type").await;
    // 直播小玩法->开发->服务端->直播能力->直播信息
    let info = sdk.info("exe启动时携带的token").await;
    // 直播小玩法->开发->服务端->直播能力->数据开放->验证签名
    let mut sign_map: std::collections::BTreeMap<&str,&str>  = std::collections::BTreeMap::new();
    sign_map.insert("x-roomid", "268");
    sign_map.insert("x-timestamp", "456789");
    sign_map.insert("x-nonce-str", "123456");
    sign_map.insert("x-msg-type", "live_gift");
    let str = sdk.verify_sign(sign_map,"abc123你好","123abc");
    // --------工具函数--------
    let random_str = make_random_string(); // 生成随机字符串
    let ts = get_now_timestamp(false); // 获取当前时间戳
    let mut sign_map: std::collections::BTreeMap<&str,&str>  = std::collections::BTreeMap::new();
    sign_map.insert("x-roomid", "268");
    sign_map.insert("x-timestamp", "456789");
    sign_map.insert("x-nonce-str", "123456");
    sign_map.insert("x-msg-type", "live_gift");
    let header_str = map_2_str(sign_map); // 请求头字符串拼接
    println!("header_str: {:#?}", header_str);
    // -------输出------
    println!("start_res: {:#?}", start_res);
    println!("stop_res: {:#?}", stop_res);
    println!("status_res: {:#?}", status_res);
    println!("info: {:#?}", info);
    println!("token: {:#?}", access_token);
    println!("random_str: {:#?}", random_str);
    println!("ts: {:#?}", ts);
    println!("verify_sign: {:#?} | (PDcKhdlsrKEJif6uMKD2dw==)", str);
}
