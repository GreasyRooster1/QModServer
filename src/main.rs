use std::fs::File;
use std::{fs, io};
use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::string::ToString;
use zip::write::{ExtendedFileOptions, FileOptions, SimpleFileOptions};
use zip::{CompressionMethod, ZipWriter};
use QModServer::ThreadPool;
use url::Url;

pub const MODPACK_FOLDER:&str = "modpacks";
pub const ZIP_TEMP_FOLDER:&str = "temp/zip";
pub const ZIP_NAME:&str = "zip.zip";

pub const THREAD_POOL_SIZE:usize = 32;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(THREAD_POOL_SIZE);

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(_) => {
                stream.unwrap()
            }
            Err(_) => {
                println!("error occurred when unwrapping stream");
                continue;
            }
        };
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
    .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let get_line = http_request[0].clone();
    let end = get_line.len()-9;
    let uri = Url::parse(&get_line[4..end]);

    let chunks = uri.split('/').collect::<Vec<&str>>();

    println!("{:?}", chunks);

    if chunks.len() < 3 {
        println!("uri malformed");
        return;
    }

    let modpack_id = chunks[1];
    let file_chunk = chunks[2];

    let modpack_path = match check_modpack_folder(modpack_id.to_string()) {
        Ok(path) => PathBuf::from(path),
        Err(_) => {
            respond_to_request(&stream,"PACK NOT FOUND".to_string());
            return;
        }
    };

    if file_chunk=="metadata"{
        let paths = fs::read_dir(modpack_path).unwrap();
        let mut response_vec = vec![];
        for path in paths {
            response_vec.push(path.unwrap().file_name().to_str().unwrap().to_string());
        }
        respond_to_request(&stream,response_vec.join("\n"));
        return;
    }

    let jar_path = modpack_path.join(file_chunk);
    println!("jar_path: {:?}", jar_path);
    let binding = fs::read(jar_path).unwrap();
    let jar_bytes = binding.as_slice();

    respond_bytes(&stream,jar_bytes)
}

fn respond_bytes(mut stream:&TcpStream,content:&[u8]){
    let header = format!("HTTP/1.0 200 OK\r\nContent-Type: application/zip\r\nContent-Length: {}\r\n\r\n",
                           content.len(),
    );

    let header_bytes = header.as_bytes();
    match stream.write_all(header_bytes).unwrap(){
        Ok(..)=>{}
        Err(error)=>{
            println!("Error occurred on stream.write: {error}");
        }
    }
    match stream.write_all(content){
        Ok(..)=>{}
        Err(error)=>{
            println!("Error occurred on stream.write: {error}");
        }
    }
    match stream.flush(){
        Ok(..)=>{}
        Err(error)=>{
            println!("Error occurred on stream.flush: {error}");
        }
    }
}

fn respond_to_request(mut stream: &TcpStream,content:String){
    let response = format!("HTTP/1.1 200 OK\r\n\r\n{content}");

    println!("{}", response);

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn check_modpack_folder(modpack:String) -> Result<String,()>{
    let path = Path::new(MODPACK_FOLDER).join(&modpack);
    if path.exists() {
        return Ok(path.to_str().unwrap().to_string());
    }
    return Err(());
}