use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipWriter};

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
    println!("{}", uri)

}

fn zip_folder(folder_path:String,output_path:String,filename:String) -> Result<(), Box<dyn std::error::Error>> {
    let zip_file_path = Path::new(&folder_path);
    let zip_file = File::create(&(output_path+ &*filename))?;

    let mut zip = ZipWriter::new(zip_file);

    // Define the files you want to compress.
    let files_to_compress: Vec<PathBuf> = vec![
        PathBuf::from("exampleImage.png"),
        PathBuf::from(".gitignore"),
        // Add more files as needed
    ];

    // Set compression options (e.g., compression method)
    let options = FileOptions::default()
        .compression_method(CompressionMethod::DEFLATE);

    // Iterate through the files and add them to the ZIP archive.
    for file_path in &files_to_compress {

        let file = File::open(file_path)?;
        let file_name = file_path.file_name().unwrap().to_str().unwrap();

        // Adding the file to the ZIP archive.
        zip.start_file(file_name, options)?;

        let mut buffer = Vec::new();
        io::copy(&mut file.take(usize::MAX), &mut buffer)?;

        zip.write_all(&buffer)?;
    }

    zip.finish()?;

    println!("Files compressed successfully to {:?}", zip_file_path);

    Ok(())
}