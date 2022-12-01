use std::{
    cmp::min,
    collections::HashMap,
    io::{Read, Seek, SeekFrom},
};

use clap::Parser;
use tracing::{debug, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    list: bool,

    /// Read from this file
    zip_file: String,

    /// Read those files
    file_list: Vec<String>,
}

fn main() {
    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::INFO)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let cli = Cli::parse();
    debug!("cli: {:?}", cli);
    if cli.list {
        let mut read = HttpReader::new(String::from(cli.zip_file));
        list_zip_contents(read);
        return;
    } else {
        let mut read = HttpReader::new(String::from(cli.zip_file));
        cat_zip_file(read, cli.file_list);
        return;
    }
}

struct HttpReader {
    url: String,
    total_len: Option<usize>,
    cur_pos: usize,
    buffer: HashMap<usize, Vec<u8>>,
}

impl HttpReader {
    pub fn new(url: String) -> HttpReader {
        HttpReader {
            url: url,
            total_len: None,
            cur_pos: 0,
            buffer: HashMap::new(),
        }
    }

    fn get_bytes(&mut self, cache_index: usize) {
        let range_start = cache_index;
        let mut range_end = range_start + 512 - 1;
        range_end = min(range_end, self.total_len.unwrap() - 1);
        let response = reqwest::blocking::Client::new()
            .get(self.url.clone())
            .header("Range", format!("bytes={}-{}", range_start, range_end))
            .send();
        let resp = response.unwrap();
        let range1 = resp.headers().get("Content-Range");
        let range2 = range1.unwrap().to_str().expect("get Content-Range error");
        let start_pos = range2.find("/").unwrap();
        let str = &range2[start_pos + 1..];
        let t_size: usize = str.parse().unwrap();
        self.total_len = Some(t_size);
        let bytes = resp.bytes().unwrap();
        let mut buf: Vec<u8> = Vec::new();
        for ele in bytes {
            buf.push(ele);
        }
        self.buffer.insert(cache_index, buf);
        debug!("read bytes={}-{} bytes", range_start, range_end,);
    }
}

impl Read for HttpReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        for pos in self.cur_pos..(self.cur_pos + buf.len()) {
            let cache_index = pos / 512 * 512;
            if !self.buffer.contains_key(&cache_index) {
                self.get_bytes(cache_index);
            }
            let data = self.buffer.get(&cache_index).unwrap();
            buf[pos - self.cur_pos] = data[pos - cache_index];
        }
        self.cur_pos += buf.len();
        return Ok(buf.len());
    }
}

impl Seek for HttpReader {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        if self.total_len.is_none() {
            let response = reqwest::blocking::Client::new()
                .head(self.url.clone())
                .send();
            let resp = response.unwrap();
            let range1 = resp.headers().get("Content-Length");
            let range2 = range1.unwrap().to_str().unwrap();
            let t_size: usize = range2.parse().unwrap();
            self.total_len = Some(t_size);
        }
        match pos {
            SeekFrom::Start(offset) => self.cur_pos = offset as usize,
            SeekFrom::End(offset) => {
                self.cur_pos = (self.total_len.unwrap() as i64 + offset) as usize
            }
            SeekFrom::Current(offset) => self.cur_pos += offset as usize,
        }
        debug!("seek {:?}, self.cur_pos = {}", pos, self.cur_pos);
        return Ok(self.cur_pos as u64);
    }
}

fn cat_zip_file(reader: impl Read + Seek, file_list: Vec<String>) {
    let mut zip = zip::ZipArchive::new(reader).unwrap();
    for filename in &file_list {
        let mut zfile = zip.by_name(&filename).unwrap();
        let mut file_content = String::new();
        zfile.read_to_string(&mut file_content).unwrap();
        if &file_list.len() > &1 {
            println!("name: {}", zfile.name());
        }
        print!("{}", file_content);
    }
}

fn list_zip_contents(reader: impl Read + Seek) -> zip::result::ZipResult<()> {
    let mut zip = zip::ZipArchive::new(reader)?;

    println!(
        "{: ^9}  {: ^10} {: ^5}   {: ^4}",
        "Length", "Date", "Time", "Name"
    );
    println!(
        "{: ^9}  {: ^10} {: ^5}   {: ^4}",
        "-".repeat(9),
        "-".repeat(10),
        "-".repeat(5),
        "-".repeat(4)
    );
    let mut total_len = 0;
    let mut total_cnt = 0;
    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;

        println!(
            "{: >9}  {:0>4}-{:0>2}-{:0>2} {:0>2}:{:0>2}   {:}",
            file.size(),
            file.last_modified().year(),
            file.last_modified().month(),
            file.last_modified().day(),
            file.last_modified().hour(),
            file.last_modified().minute(),
            file.name(),
        );
        total_len += file.size();
        total_cnt += 1;
    }
    println!("{}{}{}", "-".repeat(9), " ".repeat(21), "-".repeat(7));
    println!("{:>9}{}{} files", total_len, " ".repeat(21), total_cnt);

    Ok(())
}
