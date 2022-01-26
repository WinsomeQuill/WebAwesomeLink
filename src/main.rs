use std::{thread, time, process::Command};

use rand::{thread_rng, Rng};
use thirtyfour::{prelude::*, WindowHandle};
use tokio;

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    let mut current_window_handle: WindowHandle;
    let mut history: Vec<String> = Vec::new();
    let mut score: usize = 0;
    let mut errors_2: u16 = 0;
    let mut web_url: String = String::from("https://www.google.com");

    println!("Web Awesome Link - это программа которая нажимает на первые попавшиеся под руку ссылки (рандомно конечно) на сайте. Я её сделал ради прикола, так-как мне было скучно, а узнать, куда сможет залесть компьютер - стало интересно!");
    println!("Программа запоминает последнии 50 ссылок, что позволяет избегать пустых страниц, где нету ссылок. В таком случае, программа выберит случайную ссылку из истории и перейдет по ней.");
    println!("Я позаботился о ваших плашках ОЗУ, поэтому если количество вкладок станет больше 5, то будет очистка, при которой все вкладки за исключением активной, будут закрыты.");
    println!("Автор - WinsomeQuill (https://vk.com/winsomequill)");
    println!("Введи адрес сайта (поумолчанию: https://google.com):");
    let a = read_line();
    if a.len() >= 3 {
        web_url = a;
    } else {
        println!("Пусто адрес! Ставлю https://google.com и запускаю программу!");
    }

    println!("Запуск! Теперь можете свернуть веб-драйвер (откроется как новый chrome) и скрипт! Желаю приятного лазанья по интернет-паутине :)");
    
    thread::spawn(|| {
        let path = format!("{}\\chromedriver.exe", std::env::current_dir().unwrap().display());
        Command::new("cmd")
                .args(&["/C", &path, "--port=4444"])
                .output()
                .expect("failed to start web driver");
    });

    thread::sleep(time::Duration::from_secs(2));
            
    let mut caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:4444", &caps).await?;
    let _ = caps.add_chrome_option("download_restrictions", 3);

    match driver.get(&web_url).await {
        Ok(r) => { println!("Success connecting to site!"); r },
        Err(e) => { panic!("Error connecting to site => {}", e) },
    };

    history.insert(score, driver.current_url().await?);
    score += 1;

    loop {
        driver.refresh().await?;

        current_window_handle = match driver.current_window_handle().await {
            Ok(handle) => handle,
            Err(_) => { continue; },
        };

        let windows = match driver.window_handles().await {
            Ok(handle) => handle,
            Err(_) => { continue; },
        };

        if windows.len() > 5 {
            println!("[Clear] Go clear windows!");
            for window in windows {
                if window != current_window_handle {
                    match driver.switch_to().window(&window).await {
                        Ok(_) => println!("[Clear] Go to other window..."),
                        Err(e) => println!("Error clear -> {}", e),
                    };

                    match driver.close().await{
                        Ok(_) => { println!("[Clear] Close window!"); continue; },
                        Err(e) => println!("Error close -> {}", e),
                    };
                }
            }
            driver.switch_to().window(&current_window_handle).await?;
        }

        match driver.find_elements(By::XPath("//a[@href]")).await {
            Ok(element) => {
                if element.len() != 0 {
                    let mut rng = thread_rng();
                    let r = rng.gen_range(0..element.len());
                    let link = match element[r].get_attribute("href").await {
                        Ok(value) => value.unwrap(),
                        Err(_) => {
                            println!("[Error] Go to back!"); 
                            driver.back().await?;
                            thread::sleep(time::Duration::from_secs(5));
                            continue;
                        },
                    };

                    match element[r].click().await {
                        Ok(_) => {
                            if score >= 50 {
                                score = 2;
                            } else {
                                score += 1;
                                if driver.current_url().await? != history[score-2] {
                                    history.insert(score-2, driver.current_url().await?);
                                    println!("[Debug] Links in History: {}", score);
                                    let _ = match driver.get(&link).await {
                                        Ok(_) => {
                                            let text = match element[r].text().await {
                                                Ok(r) => r,
                                                Err(_) => continue,
                                            };
                                            println!("Go -> {} | Link: {}", text, link);
                                            errors_2 = 0;
                                            driver.switch_to().window(&driver.current_window_handle().await?).await?;
                                            thread::sleep(time::Duration::from_secs(5));
                                            continue;
                                        },
                                        Err(_) => {
                                            thread::sleep(time::Duration::from_millis(100));
                                            continue;
                                        },
                                    };
                                } else {
                                    score -= 1;
                                }
                            }
                        },
                        Err(_) => {
                            errors_2 += 1;
                            thread::sleep(time::Duration::from_millis(100));
                            match errors_2 {
                                5..=10 => {
                                    errors_2 = 0;
                                    driver.back().await?;
                                },
                                _ => {},
                            };
                            continue;
                        },
                    };
                }

                let mut rng = thread_rng();
                let r = rng.gen_range(0..history.len());
                println!("Back to history Link: {}", history[r]);
                match driver.get(&history[r]).await {
                    Ok(_) => { thread::sleep(time::Duration::from_secs(5)); driver.refresh().await?; continue; },
                    Err(_) => { thread::sleep(time::Duration::from_millis(100)); driver.refresh().await?; continue; },
                };
            },
            Err(_) => {
                thread::sleep(time::Duration::from_secs(5));
                continue;
            },
        };
    }
}

fn read_line() -> String {
    let mut buffer: String = String::new();
    let stdin = std::io::stdin();
    stdin.read_line(&mut buffer).unwrap();
    buffer.trim().to_string()
}