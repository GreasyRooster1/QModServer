use std::fs::File;
use std::{fs, io};
use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::string::ToString;
use zip::write::{ExtendedFileOptions, FileOptions, SimpleFileOptions};
use zip::{CompressionMethod, ZipWriter};

pub const MODPACK_FOLDER:&str = "modpacks";
pub const ZIP_TEMP_FOLDER:&str = "temp/zip";
pub const ZIP_NAME:&str = "zip.zip";

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
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
    let uri = &get_line[4..end];

    let modpack_id = uri.replace("/","");


    let path = match check_modpack_folder(modpack_id) {
        Ok(path) => path,
        Err(_) => {
            respond_to_request(&stream,"PACK NOT FOUND".to_string());
            return;
        }
    };
    
    match zip_folder(path,ZIP_TEMP_FOLDER.to_string(),ZIP_NAME.to_string()) {
        Ok(_) => {}
        Err(err) => {
            respond_to_request(&stream,format!("ERROR WITH ZIP: {0}",err.to_string()))
        }
    }

    let zip_path = Path::new(ZIP_TEMP_FOLDER).join(ZIP_NAME);
    let binding = fs::read(zip_path).unwrap();
    let zip_bytes = binding.as_slice();

    respond_bytes(&stream,zip_bytes)
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

fn zip_folder(folder_path:String,output_path:String,filename:String) -> Result<(), Box<dyn std::error::Error>> {

    let zip_file_path = Path::new(&output_path);
    let zip_file = File::create(Path::new(&output_path).join(filename))?;

    let mut zip = ZipWriter::new(zip_file);


    let paths = fs::read_dir(&folder_path)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect::<Vec<_>>();

    // Set compression options (e.g., compression method)
    let options:FileOptions<'_,()> = FileOptions::default()
        .compression_method(CompressionMethod::DEFLATE);

    // Iterate through the files and add them to the ZIP archive.
    for file_path in &paths {

        let file_name = file_path.file_name().unwrap().to_str().unwrap();

        // Adding the file to the ZIP archive.
        zip.start_file(file_name, options)?;

        let buffer:Vec<u8> = fs::read(file_path)?;
        zip.write_all(&buffer)?;
    }

    zip.finish()?;

    println!("Files compressed successfully to {:?}", zip_file_path);

    Ok(())
}