//! 抖音的开放平台的第三方SDK
//! 
//! # 说明
//! 
//! 这是一个，个人（被动）维护的，的抖音开放平台的RustSDK项目
//! 仅限使用到的API进行封装，不包含其他API，后续可能会逐步增加其他API
//! 如果您需要对这个SDK进行扩展，包括项目结构调整，提供功能说明，或添加测试用例，提交PR，或发送邮件。稍后我会对项目进行更新。
//! 
//! 对应的 抖音官方文档为[https://developer.open-douyin.com/docs/resource/zh-CN/interaction/develop/server/server-api-introduction]()
//! 
//! 我刚看到抖音的服务端分为很多版块，这里只是直播小玩法的服务端文档，由于对文档不熟悉，不知道如何抽取公共功能。针对版本号~0.1，仅对其扩展不做破坏性变更
//! 
//! 这只是一个练手项目，对于rust我还有很多困惑，也不知道如何精简项目，欢迎对Rust进行学习和交流。
//! 
//! # Example
//! ```rust
//!     let app_private_key = include_str!("private_key.pem");
//!     let config = DouyinConfig {
//!         appid: "appid",
//!         secret: "secret",
//!         app_private_key: app_private_key,
//!         ..Default::default()
//!     };
//!     let mut sdk = SDK::new(config);
//! 
//!     // 直播小玩法->开发->服务端->接口调用凭证->getAccessToken->获取access_token
//!     let token = sdk.get_access_token().await;
//!     // 直播小玩法->开发->服务端->直播能力->数据开放->启动任务
//!     let start_res = sdk.task::<LiveOpenReqDataStart>("start","roomid","appid","msg_type").await;
//!     // 直播小玩法->开发->服务端->直播能力->数据开放->停止任务
//!     let stop_res = sdk.task::<LiveOpenReqDataStop>("stop","roomid","appid","msg_type").await;
//!     // 直播小玩法->开发->服务端->直播能力->数据开放->查询任务状态
//!     let status_res = sdk.task::<LiveOpenReqDataStatus>("status","roomid","appid","msg_type").await;
//!     // 直播小玩法->开发->服务端->直播能力->直播信息
//!     let info = sdk.info("exe启动时携带的token").await;
//! ````
pub mod sign;


use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::{
    fs::{read_to_string, File},
    io::{
        AsyncWriteExt , // for write_all()
        BufWriter
    }
};
use rand::Rng;

/*
* SDK 的结构
*/
#[derive(Debug)]
pub struct SDK {
    pub appid: String, // appid
    pub secret: String, // secret
    pub app_private_key: String,
    pub pkcs_type: sign::PkcsType, // app的私钥
    pub base_url: String, // 请求数据的url
    pub access_base_url: String, // 获取access_token的url
    pub access_token_cache_file_path: String, // access_token缓存文件路径，当进程重启后优先读取文件缓存

    pub access_token: String, // access_token
    pub expires_in: u64, // access_token的过期时间
}

/*
* 初始化时的参数
*/
#[derive(Default)]
pub struct DouyinConfig<'a> {
    pub appid: &'a str,
    pub secret: &'a str,
    pub app_private_key: &'a str,
    pub pkcs_type: Option<sign::PkcsType>,
    pub base_url:Option<&'a str>,
    pub access_base_url:Option<&'a str>,
    pub access_token_cache_file_path: Option<&'a str>,
}

/*
* get_access_token 的返回结构体
*/
#[derive(Deserialize, Serialize,Debug,Clone)]
pub struct AccessTokenRes {
    err_no: i32,
    err_tips: String,
    data: AccessTokenResData
}
#[derive(Deserialize, Serialize,Debug,Clone)]
pub struct AccessTokenResData {
    access_token: String,
    expires_in: u64
}

#[derive(Deserialize, Serialize,Debug,Clone)]
pub struct LiveOpenRes<T> {
    err_no: i32,
    err_msg: String,
    logid:String,
    data:T
}
#[derive(Deserialize, Serialize,Debug,Clone)]
pub enum LiveOpenReqDataEnum {
    Start(LiveOpenReqDataStart),
    Stop(LiveOpenReqDataStop),
    Status(LiveOpenReqDataStatus),
}
#[derive(Deserialize, Serialize,Debug,Clone)]
pub struct LiveOpenReqDataStart {
    pub task_id:String,
}
#[derive(Deserialize, Serialize,Debug,Clone)]
pub struct LiveOpenReqDataStop {
}


#[derive(Debug, Serialize,Deserialize)]
pub struct RoomInfo {
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    pub data: Option<RoomInfoData>,
}

#[derive(Debug,Serialize, Deserialize)]
pub struct RoomInfoData {
    pub room_id: u64,
    pub anchor_open_id: String,
    pub avatar_url: String,
    pub nick_name: String,
}

#[derive(Deserialize, Serialize,Debug,Clone)]
pub struct LiveOpenReqDataStatus {
    pub status: u8, // 1 任务不存在 2任务未启动 3任务运行中
}
impl SDK  {
    /**
     * 构造函数初始化实例
     */
    pub fn new (config: DouyinConfig) -> Self {
        // 将传入的参数和对象的默认值合并
        SDK {
            appid: String::from(config.appid),
            secret: String::from(config.secret),
            app_private_key: String::from(config.app_private_key),
            access_base_url: config.access_base_url.unwrap_or("https://developer.toutiao.com").to_owned(),
            base_url:  config.base_url.unwrap_or("https://webcast.bytedance.com").to_owned(),
            pkcs_type: config.pkcs_type.unwrap_or(sign::PkcsType::Pkcs1),

            access_token: "".to_string(),
            access_token_cache_file_path: config.access_token_cache_file_path.unwrap_or(Self::get_exe_path("douyin_access_token.json").to_str().unwrap()).to_string(),
            expires_in: 0,
        }
    }

    /**
     * 获取exe的路径，用来保存access_token缓存文件。
     */
    fn get_exe_path(name: &str) -> std::path::PathBuf {
        let current_exe_path = std::env::current_exe().unwrap();
        let mut path = std::path::PathBuf::from(&current_exe_path);
        path.pop();
        path.push(name);
        path
    }

    /**
     * 请求失败后更新下次请求时间戳，避免频繁请求。
     * 因为正常情况下token有7200秒的过期时间，此处为3600秒后开始更新token，正常情况下在剩余的一个小时内失败几次无所谓
     * 只要成功一次即可，这里设计为超过3600秒后，如果请求失败，每1分钟请求一次。
     */
    fn update_expires_fallback_time(&mut self) {
        self.expires_in = get_now_timestamp(false) + 60;
    }

    /**
     * 获取access_token
     */ 
    pub async fn get_access_token (&mut self) -> Result<String,Box<dyn std::error::Error>> {
        let ts: u64 = get_now_timestamp(false);
        // 如果已过期
        if self.expires_in < ts {
            // 读取文件缓存，判断是否过期
            let read_data = Self::read_file(&self.access_token_cache_file_path).await;
            // 如果文件缓存过期
            if read_data.expires_in < ts {
                let res: Result<reqwest::Response, reqwest::Error>= self.access_token_request("/api/apps/v2/token",
                    json!({
                        "appid":self.appid,
                        "secret":self.secret,
                        "grant_type":"client_credential" // 获取 access_token 时值为 client_credential
                    })
                ).await;
                if let Ok(resopnse) = res {
                    let access_token_res = resopnse.json::<AccessTokenRes>().await;
                    if let Ok(data) = access_token_res {
                        if data.err_no == 0 {
                            // 如果过期时间大于1小时，则缩短到一小时，否则直接使用过期时间
                            let expires;
                            if data.data.expires_in > 3600 {
                                expires = 3600 + ts;
                            } else {
                                expires = data.data.expires_in + ts;
                            }
                            let token = data.data.access_token;
                            // 写入文件
                            Self::write_file(AccessTokenResData {
                                expires_in: expires,
                                access_token: token.clone()
                            }, &self.access_token_cache_file_path).await?;
                            self.access_token = token;
                            self.expires_in = expires;
                        } else {
                            // -1 系统错误
                            // 40015 appid 错误
                            // 40017 secret 错误
                            // 40020 grant_type 不是 client_credential
                            // 其它 参数为空
                            self.update_expires_fallback_time();
                            return Err(format!("获取access_token失败,err_no:{}, err_msg: {}",data.err_no, data.err_tips).into());
                        }
                    } else {
                        self.update_expires_fallback_time();
                        return Err("json解析失败".into());
                    }
                } else {
                    self.update_expires_fallback_time();
                    return Err("请求失败".into());
                }
            } else {
                // 把文件的过期时间写入内存
                self.access_token = read_data.access_token;
                self.expires_in = read_data.expires_in;
            }
        }
        return Ok(self.access_token.clone());
    }

    /**
     * 写入access_token到缓存文件
     */
    async fn write_file(data:AccessTokenResData, file_name: &str) -> Result<(), std::io::Error> {
        let path: std::path::PathBuf = Self::get_exe_path(file_name);
        let file = File::create(&path).await;
        match file {
            Ok(f) => {
                let mut writer = BufWriter::new(f);
                let json_str = serde_json::to_string(&data);
                match json_str {
                    Ok(str) => {
                        let _ = writer.write_all(str.as_bytes()).await;
                        let _ = writer.flush().await;
                        println!("写入文件成功");
                        return Ok(());
                    },
                    Err(err) => {
                        println!("toml解析失败:{:?}",err);
                        return Ok(());
                    }
                }
            },
            Err(err) => {
                println!("写入文件失败:{:?}",err);
                return Ok(());
            }
        }
    }

    /**
     * 读取access_token文件
     */
    async fn read_file(file_name: &str) -> AccessTokenResData {
        let path = Self::get_exe_path(file_name);
        let contents= read_to_string(&path).await;
        if let Ok(json_text) = contents{
            let result: Result<AccessTokenResData, serde_json::Error> = serde_json::from_str(&json_text);
            if let Ok(data) = result {
                return data;
            } else if let Err(err) = result {
                println!("Info: json解析错误10分钟后重试: {:#?}", err);
            }
        } else if let Err(err) = contents {
            println!("Info: token缓存文件不存在-程序继续执行, : {:?}", err);
        }
        // 读取失败，返回默认值，稍后转到请求数据
        return AccessTokenResData{
            access_token: "".to_string(),
            expires_in: 0,
        }
    }
    async fn access_token_request(&self,path:&str,map:Value) -> Result<reqwest::Response, reqwest::Error> {
        let client = reqwest::Client::new();
        let res = client.post(format!("{}{}", self.access_base_url , path))
            .header("Accept", "application/json")
            .json(&map) 
            .send()
            .await;
        return res;
    }
    pub async fn sign_request(&mut self,path:&str,map:Value) -> Result<reqwest::Response, reqwest::Error> {
        let http_method = "POST";
        let timestamp = get_now_timestamp(false).to_string();
        let random_string = make_random_string();
        let json_str = serde_json::to_string(&map).unwrap();
        let sign_str = format!(
            "{}\n{}\n{}\n{}\n{}\n",
            http_method, path, timestamp, random_string, json_str
        );
        
        let base64_str  = sign::sign_base64(sign_str.as_bytes(),&self.app_private_key,&sign::PkcsType::Pkcs8);
    
        let byte_authorization = format!(
            "SHA256-RSA2048 appid=\"{}\",nonce_str=\"{}\",timestamp=\"{}\",key_version=\"1\",signature=\"{}\"",
            self.appid, random_string, timestamp,base64_str
        );
        let client = reqwest::Client::new();
        let access_token = self.get_access_token().await.expect("获取access_token失败");
        let res = client.post(format!("{}{}", self.base_url, path))
            .header("Byte-Authorization", byte_authorization)
            .header("Accept", "application/json")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header("access-token", access_token)
            .json(&map) 
            .send()
            .await;
        return res;
    }

    
    pub async fn access_request(&mut self,path:&str,map:Value) -> Result<reqwest::Response, reqwest::Error> {
        let client = reqwest::Client::new();
        let access_token = self.get_access_token().await.expect("获取access_token失败");
        let res = client.post(format!("{}{}", self.base_url, path))
            .header("Accept", "application/json")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header("X-Token", access_token)
            .json(&map) 
            .send()
            .await;
        return res;
    }
    /**
     * post请求task 
     */
    pub async fn task<T>(&mut self,task:&str,roomid:&str,appid:&str,msg_type:&str) -> Result<LiveOpenRes<T>, reqwest::Error>
        where T: DeserializeOwned
    {
        let path = format!("/api/live_data/task/{}",task);
        let result: Result<reqwest::Response, reqwest::Error> = self.sign_request(&path[..],json!({"roomid":roomid,"appid":appid,"msg_type":msg_type})).await;
        let res = result?.json::<LiveOpenRes<T>>().await;
        return res;
    }

    /**
     * 使用 access_token 获取 直播间信息
     */
    pub async fn info(&mut self,token:&str) -> Result<RoomInfo, reqwest::Error> {
        let result: Result<reqwest::Response, reqwest::Error> = self.access_request("/api/webcastmate/info",json!({"token":token})).await;
        let res = result?.json::<RoomInfo>().await;
        return res;
    }
}



/**
 * 生成随机字符串
 */
pub fn make_random_string() -> String {
    let mut rng = rand::thread_rng();
    let random_numbers: Vec<u32> = (0..4).map(|_| rng.gen()).collect();
    let mut formatted_output = String::new();
    for chunk in random_numbers.chunks(1) {
        for num in chunk {
            formatted_output.push_str(&format!("{:08X}", num));
        }
    }
    formatted_output
}
/**
 * 获取当前时间戳，单位为秒或者毫秒。
 * @param ms 是否为毫秒
 */
pub fn get_now_timestamp(ms: bool) -> u64 {
    let now = std::time::SystemTime::now();
    let since_epoch = now.duration_since(std::time::UNIX_EPOCH).expect("时光倒流");
    // 获取毫秒级时间戳
    if ms {
        return since_epoch.as_secs() * 1000 + since_epoch.subsec_millis() as u64;
    } else {
        return since_epoch.as_secs()
    }
}