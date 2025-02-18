use serde_json::Value;
use std::error::Error;
use std::fs;
use std::io::{self, Read, Write}; // 引入 io::stdin
use std::path::Path;

fn convert_file(path: &Path) -> Result<(), Box<dyn Error>> {
    let mut file = fs::File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut value: Value = serde_json::from_str(&contents)?;
    rebuild_url(&mut value);

    let mut file = fs::File::create(path)?;
    let formatted_json = serde_json::to_string(&value)?;
    file.write_all(formatted_json.as_bytes())?;

    println!("转换文件成功: {:?}", path);
    Ok(())
}

fn rebuild_url(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if let Some(uri_value) = map.get_mut("uri") {
                if let Value::String(uri_str) = uri_value {
                    *uri_str = uri_str.replace('+', "%2B");
                }
            }
            for (_key, val) in map.iter_mut() {
                rebuild_url(val);
            }
        }
        Value::Array(arr) => {
            for val in arr.iter_mut() {
                rebuild_url(val);
            }
        }
        _ => {}
    }
}

fn process_directory(path: &Path) -> Result<(), Box<dyn Error>> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                process_directory(&path)?;
            } else if let Some(ext) = path.extension() {
                if ext == "json" {
                    convert_file(&path)?;
                }
            }
        }
    }
    Ok(())
}

fn main() {
    // 获取当前工作目录
    let current_dir = match std::env::current_dir() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("获取当前目录失败: {}", err);
            return;
        }
    };

    println!("当前工作目录: {:?}", current_dir);
    println!("是否要在此目录及其子目录中转换 JSON 文件？ (yes/no)");

    // 读取用户输入
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            //去掉末尾的换行符
            input = input.trim().to_string();
            if input.to_lowercase() == "yes" {
                // 转换为小写进行比较

                // 处理转换
                if let Err(err) = process_directory(&current_dir) {
                    eprintln!("发生错误: {}", err);
                    let mut current_err: Option<&dyn Error> = Some(err.as_ref());
                    while let Some(source) = current_err {
                        eprintln!("Caused by: {}", source);
                        current_err = source.source();
                    }
                } else {
                    println!("*.json 文件转换完成!");
                    println!("请按任意键退出...");
                    let _ = io::stdin().read(&mut [0u8]).unwrap();
                }
            } else if input.to_lowercase() == "no" {
                println!("已取消操作。");
            } else {
                println!("无效的输入。请输入 'yes' 或 'no'。");
            }
        }
        Err(err) => {
            eprintln!("读取输入时出错: {}", err);
        }
    }
}
