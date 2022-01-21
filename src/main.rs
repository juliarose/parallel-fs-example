use std::sync::Arc;
use std::path::Path;
use tokio::task::JoinHandle;
use std::collections::HashMap;
use futures::future::join_all;

struct FileContents {
    filename: String,
    contents: String,
}

async fn read_file(filename: String) -> Result<FileContents, &'static str> {
    let rootdir = env!("CARGO_MANIFEST_DIR");
    
    if let Some(filepath) = Path::new(rootdir).join(format!("files/{}.txt", filename)).to_str() {
        if let Ok(contents) = async_fs::read_to_string(filepath).await {
            Ok(FileContents {
                filename,
                contents,
            })
        } else {
            Err("file does not exist")
        }
    } else {
        Err("could not coerce filepath to utf8 string")
    }
}

async fn read_files(filenames: &Vec<String>) -> Vec<Result<FileContents, &'static str>> {
    let mut tasks: Vec<JoinHandle<Result<FileContents, &'static str>>>= vec![];
    
    for filename in filenames {
        // must be cloned to move across threads
        let filename = filename.clone();
        
        tasks.push(tokio::spawn(async move {
            read_file(filename).await
        }));
    }
    
    let mut results: Vec<Result<FileContents, &'static str>> = Vec::new();
    
    for join_result in join_all(tasks).await {
        results.push(match join_result {
            Ok(task_result) => task_result,
            Err(_err) => Err("join error"),
        })
    }
    
    results
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut data: HashMap<String, Arc<String>> = HashMap::new();
    let filenames = vec![
        String::from("cat"),
        String::from("tokyo"),
        String::from("banana"),
    ];
    
    for result in read_files(&filenames).await {
        match result {
            Ok(file_contents) => {
                data.insert(file_contents.filename, Arc::new(file_contents.contents));
            },
            Err(err) => println!("{}", err),
        }
    }
    
    println!("Data: {:?}", data);
    
    Ok(())
}
