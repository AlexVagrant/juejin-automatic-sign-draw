#[allow(dead_code)]
use config::Config;
use reqwest::header::{HeaderMap, CONTENT_TYPE, COOKIE, USER_AGENT};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;

// "{\"err_no\":0,\"err_msg\":\"success\",\"data\":true}"
#[derive(Deserialize, Debug)]
struct JuejinResponse {
    err_no: i32,
    err_msg: String,
    data: bool,
}

#[derive(Deserialize, Debug)]
struct DrawStatus {
    err_no: i32,
    err_msg: String,
    data: DrawStatusData,
}
#[derive(Deserialize, Debug)]
struct DrawStatusData {
    free_count: i32,
}

#[derive(Deserialize, Debug)]
struct SignResult {
    err_no: i32,
    err_msg: String,
    data: SignResultData,
}

#[derive(Deserialize, Debug)]
struct SignResultData {
    incr_point: i32,
    sum_point: i32,
}

#[derive(Deserialize, Debug)]
struct DrawResult {
    err_no: i32,
    err_msg: String,
    data: DrawResultData,
}

#[derive(Deserialize, Debug)]
struct DrawResultData {
    lottery_name: String,
}

async fn push(
    content: String,
    setting: HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let title = String::from("签到结果");
    let mut map = HashMap::new();
    map.insert("token", setting.get("token").unwrap());
    map.insert("title", &title);
    map.insert("content", &content);
    let client = reqwest::Client::new();
    client
        .post(setting.get("push_base_url").unwrap())
        .json(&map)
        .send()
        .await?;

    Ok(())
}

struct JueJin<'a> {
    client: &'a Client,
    base_url: &'a String,
}

impl JueJin<'_> {
    fn new<'a>(client: &'a Client, base_url: &'a String) -> JueJin<'a> {
        JueJin { client, base_url }
    }

    async fn get<T: for<'de> Deserialize<'de>>(
        self: &Self,
        url: &String,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let resp = self
            .client
            .get(format!("{}{}", self.base_url, url))
            .send()
            .await?;
        let body = resp.json::<T>().await?;
        Ok(body)
    }

    async fn post<T: for<'de> Deserialize<'de>>(
        self: &Self,
        url: &String,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let resp = self
            .client
            .post(format!("{}{}", self.base_url, url))
            .send()
            .await?;
        let body = resp.json::<T>().await?;
        Ok(body)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let setting = Config::builder()
        .add_source(config::File::with_name("config/juejin"))
        .add_source(config::Environment::with_prefix("APP"))
        .build()
        .unwrap()
        .try_deserialize::<HashMap<String, String>>()
        .unwrap();

    let base_url = String::from(setting.get("base_url").unwrap());

    let mut headers = HeaderMap::new();

    headers.insert(COOKIE, setting.get("cookie").unwrap().parse().unwrap());
    headers.insert(
        USER_AGENT,
        setting.get("user_agent").unwrap().parse().unwrap(),
    );
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let juejin = JueJin::new(&client, &base_url);

    let check_result: JuejinResponse = juejin
        .get::<JuejinResponse>(setting.get("check_sign").unwrap())
        .await?;

    let mut push_message = String::from("");

    if check_result.err_no != 0 {
        push_message = format!("{}\n{}", "查询签到失败", &push_message);
    } else if check_result.data {
        push_message = format!("{}\n{}", "今日已参与签到", &push_message);
    } else {
        let sign_data = juejin
            .post::<SignResult>(setting.get("sign").unwrap())
            .await?;
        push_message = format!(
            "{}\n{}",
            format!(
                "签到获得：{} 总获得{},签到成功",
                sign_data.data.incr_point, sign_data.data.sum_point
            ),
            &push_message
        );
    }

    let time_seconds = Duration::from_secs(5);
    sleep(time_seconds);

    let check_draw: DrawStatus = juejin
        .get::<DrawStatus>(setting.get("check_draw").unwrap())
        .await?;
    if check_draw.data.free_count == 0 {
        push_message = format!("{}\n{}", "今日已无免费抽奖次数", &push_message);
    } else {
        let draw_result = juejin
            .post::<DrawResult>(setting.get("draw").unwrap())
            .await?;
        push_message = format!(
            "{}\n{}",
            format!("{}{}", "恭喜抽到: ", draw_result.data.lottery_name),
            &push_message
        );
    }

    println!("{:?}", push_message);

    push(push_message, setting).await?;

    Ok(())
}
