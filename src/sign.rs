use base64::Engine;
use rsa::{pkcs1::DecodeRsaPrivateKey, pkcs8::DecodePrivateKey, Pkcs1v15Sign, RsaPrivateKey};
use rsa::sha2::{Digest, Sha256};

/**
 * 定义支持的PKCS类型
 */
#[derive(Debug)]
pub enum PkcsType {
    Pkcs8,
    Pkcs1,
}

/**
 * 单行 key 字符转多行(每行最多64个字符)
 */
fn get_key_lines(key_str: &str) -> Vec<String> {
    key_str
        .chars()
        .collect::<Vec<char>>()
        .chunks(64)
        .map(|ss| ss.iter().collect::<String>())
        .collect::<Vec<String>>()
}

/**
 * 获取pem格式的私钥字符串
 */
pub fn get_pri_pem_key_str(key_str: &str, pkcs: &PkcsType) -> String {
    let (begin, end) = match pkcs {
        PkcsType::Pkcs1 => (
            "-----BEGIN RSA PRIVATE KEY-----",
            "-----END RSA PRIVATE KEY-----",
        ),
        PkcsType::Pkcs8 => ("-----BEGIN PRIVATE KEY-----", "-----END PRIVATE KEY-----"),
    };
    let mut lines = get_key_lines(key_str);
    lines.insert(0, begin.to_string());
    lines.push(end.to_string());
    lines.join("\n")
}

/**
 * 获取PEM格式的私钥
 */
pub fn get_pri_pem_key(key_str: &str, pkcs: &PkcsType,parse:bool) -> RsaPrivateKey {
    let mut pem_key_str = key_str.to_string();
    if parse {
        pem_key_str = get_pri_pem_key_str(key_str, pkcs);
    } 
    let pem_key = match pkcs {
        PkcsType::Pkcs1 => RsaPrivateKey::from_pkcs1_pem(&pem_key_str).unwrap(),
        PkcsType::Pkcs8 => RsaPrivateKey::from_pkcs8_pem(&pem_key_str).unwrap(),
    };
    pem_key
}

/**
 * 签名
 */
pub fn sign(content: &[u8], pri_key: &str, pkcs: &PkcsType) -> Vec<u8> {
    let pem_key = get_pri_pem_key(pri_key, pkcs,false);
    // 签名的padding与hash方法一致, rsa::sha2::* 需要引入rsa crate时,添加sha2 features
    let padding: Pkcs1v15Sign = Pkcs1v15Sign::new::<rsa::sha2::Sha256>();
    // 对加签的原文进行sha2摘要,然后对摘要内容加签
    let hashed = Sha256::new().chain_update(content).finalize();
    let sig: Vec<u8> = pem_key.sign( padding, &hashed).unwrap();
    sig
}

/**
 * 签名,并对签名进行base64编码
 */
pub fn sign_base64(content: &[u8], pri_key: &str, pkcs: &PkcsType) -> String {
    let sig = sign(content, pri_key, pkcs);
    let sig_base64 = base64::engine::general_purpose::STANDARD.encode(sig);
    sig_base64
}
