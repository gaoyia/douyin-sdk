
extern crate douyin; // 导入lib.rs中的库

use douyin::{make_random_string, DouyinConfig, SDK}; // 使用lib.rs中的函数

#[tokio::main]
async fn main ()  {
    // 它将在编译时写进二进制可执行文件中。如果需要在运行时更新请自行修改。
    let app_private_key = include_str!("private_key.pem");
    let config = DouyinConfig {
        appid: "appid",
        secret: "secret",
        app_private_key: app_private_key,
        ..Default::default()
    };
    let mut sdk = SDK::new(config);
    let token = sdk.get_access_token().await;
    println!("token - {}",token.unwrap());

    println!("make_random_string() - {}", make_random_string());

    let task_res = sdk.task("start","roomid","appid","msg_type").await;
    println!("task_res ----- {:?}", task_res);

}