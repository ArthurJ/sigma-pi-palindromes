use reqwest::Client;

use std::collections::VecDeque;
use iterslide::SlideIterator;

use serde_json::Value;

use tokio::spawn;
use tokio::task::JoinHandle;
use futures::future::join_all;

#[allow(unused_imports)]
use std::time::Duration;
use chrono::prelude::*;
use futures::{stream, StreamExt};


const CONCURRENT_REQUESTS: usize = 50;
const WINDOW_SIZE: usize = 5;
const STEP_SIZE: usize = 1000 - WINDOW_SIZE +1;


#[allow(dead_code)]
#[tokio::main]
async fn main() {

    let super_step= 1_000usize;
    for epoch in 1..100_000{
        // if epoch%100==0{
        //     println!("{}..{}", (epoch-1)*super_step, super_step*epoch);
        // }
        let client = Client::new();

        let urls: Vec<String> = ((epoch-1)*super_step..=super_step*epoch)
            .step_by(STEP_SIZE)
            .map(|s|
                    format!("https://api.pi.delivery/v1/pi?start={s}&numberOfDigits=1000&radix=10"))
            .collect();

        let bodies = stream::iter(urls)
            .map(|url| {
                let client = &client;
                async move {
                    let resp = client.get(url).send().await?;
                    resp.text().await
                }})
            .buffered(CONCURRENT_REQUESTS);

        bodies
            .for_each(|res| async {
                match res{
                    Ok(ok_res) => {
                        let digits = match ok_res.parse::<Value>() {
                            Ok(mut sus) => {
                                match sus.get_mut("content"){
                                    None => {panic!("Empty Content")}
                                    Some(content) => { content.to_string() }
                                }}
                            Err(_) => {panic!("Parse Error: {}", ok_res);}
                        };
                        verify(&digits, WINDOW_SIZE);
                    }
                    Err(e) => {
                        let datetime = Local::now();
                        println!("Error @ {datetime:?}:\n\t{}", e);
                    }}}).await;
    }}

#[allow(dead_code)]
#[tokio::main]
async fn sequential_main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder().build()?;

    for s in (0..100_000_000).step_by(STEP_SIZE){
        let url =
            format!("https://api.pi.delivery/v1/pi?start={s}&numberOfDigits=1000&radix=10");
        println!("{}", url);
        let res = client.get(url).send().await?;

        let digits = res
            .text()
            .await?
            .parse::<Value>()?
            .get_mut("content").unwrap()
            .to_string();

        verify(&digits, WINDOW_SIZE);
    }
    Ok(())
}

#[allow(dead_code)]
#[tokio::main]
async fn async_main() -> Result<(), Box<dyn std::error::Error>> {

    let super_step= 1_000usize;
    for epoch in 1..100_000{
        // if epoch%100==0{
        //     println!("{}..{}", (epoch-1)*super_step, super_step*epoch);
        // }

        let mut tasks: Vec<JoinHandle<Result<(), ()>>> = vec![];
        for s in ((epoch-1)*super_step..=super_step*epoch).step_by(STEP_SIZE) {
            tasks.push(
                spawn(async move {
                    let client =
                        Client::builder()/*.timeout(Duration::from_secs(300))*/.build().unwrap();
                    let url =
                        format!("https://api.pi.delivery/v1/pi?start={s}&numberOfDigits=1000&radix=10");
                    match client.get(url.clone()).send().await {
                        Ok(res) => {
                            match res.text().await {
                                Ok(text) => {
                                    let digits = text.parse::<Value>().unwrap()
                                        .get_mut("content").unwrap()
                                        .to_string();
                                    verify(&digits, WINDOW_SIZE);
                                }
                                Err(e) => {
                                    println!("Error reading content: {}", e);
                                }}}
                        Err(e) => {
                            let datetime = Local::now();
                            println!("Error @ {datetime:?}:\n\t{}", e);
                        }};
                    Ok(())
                }));}
        join_all(tasks).await;
    }
    Ok(())
}

fn verify(digits:&str, w_size:usize){
    for window in digits.chars().collect::<Vec<char>>().slide(w_size){
        match window[0] {
            '1'|'3'|'7'|'9' => { //apenas ímpares
                if palindrome(&window) {
                    println!("{:?}", window.iter().collect::<String>())
                }},
            _ => ()
        }}}

fn palindrome(x:&VecDeque<char>) -> bool{
    let len = x.len();
    if len==1{ return true }
    for i in 0..len{
        if x[i] != x[len-i-1]{
            return false
        }}
    true
}
