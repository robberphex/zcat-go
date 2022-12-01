use std::{
    cmp::min,
    collections::HashMap,
    io::{Read, Seek, SeekFrom},
};

fn main() {
    // let mut read = HttpReader::new(String::from(
    //     "http://arms-apm-cn-hangzhou.oss-cn-hangzhou.aliyuncs.com/2.8.0-luyanbo/ArmsAgent.zip",
    // ));
    // let fname = std::path::Path::new("/tmp/ArmsAgent.zip");
    // let file = fs::File::open(fname).unwrap();
    // let mut fread = BufReader::new(file);

    // let pos = read.seek(SeekFrom::End(0)).unwrap();
    // let fpos = fread.seek(SeekFrom::End(0)).unwrap();
    // println!("pos={},fpos={}", pos, fpos);
    // let pos = read.seek(SeekFrom::Start(79501248)).unwrap();
    // let fpos = fread.seek(SeekFrom::Start(79501248)).unwrap();
    // println!("pos={},fpos={}", pos, fpos);
    // let mut buffer = [0; 4];
    // let mut fbuffer = [0; 4];
    // read.read(&mut buffer[..]);
    // fread.read(&mut fbuffer[..]);
    // println!("buffer={:?},fbuffer={:?}", buffer, fbuffer);

    // let pos = read.seek(SeekFrom::Current(16)).unwrap();
    // let fpos = fread.seek(SeekFrom::Current(16)).unwrap();
    // println!("pos={},fpos={}", pos, fpos);

    // let pos = read.seek(SeekFrom::Start(79501248)).unwrap();
    // let fpos = fread.seek(SeekFrom::Start(79501248)).unwrap();
    // println!("pos={},fpos={}", pos, fpos);

    // let mut buffer = [0; 2];
    // let mut fbuffer = [0; 2];
    // read.read(&mut buffer[..]);
    // fread.read(&mut fbuffer[..]);
    // println!("1\tbuffer={:?},fbuffer={:?}", buffer, fbuffer);

    // let mut buffer = [0; 2];
    // let mut fbuffer = [0; 2];
    // read.read(&mut buffer[..]);
    // fread.read(&mut fbuffer[..]);
    // println!("2\tbuffer={:?},fbuffer={:?}", buffer, fbuffer);

    // let mut buffer = [0; 2];
    // let mut fbuffer = [0; 2];
    // read.read(&mut buffer[..]);
    // fread.read(&mut fbuffer[..]);
    // println!("3\tbuffer={:?},fbuffer={:?}", buffer, fbuffer);

    // let mut buffer = [0; 2];
    // let mut fbuffer = [0; 2];
    // read.read(&mut buffer[..]);
    // fread.read(&mut fbuffer[..]);
    // println!("4\tbuffer={:?},fbuffer={:?}", buffer, fbuffer);

    // let mut buffer = [0; 4];
    // let mut fbuffer = [0; 4];
    // read.read(&mut buffer[..]);
    // fread.read(&mut fbuffer[..]);
    // println!("5\tbuffer={:?},fbuffer={:?}", buffer, fbuffer);

    // let mut buffer = [0; 4];
    // let mut fbuffer = [0; 4];
    // read.read(&mut buffer[..]);
    // fread.read(&mut fbuffer[..]);
    // println!("6\tbuffer={:?},fbuffer={:?}", buffer, fbuffer);

    // let mut buffer = [0; 2];
    // let mut fbuffer = [0; 2];
    // read.read(&mut buffer[..]);
    // fread.read(&mut fbuffer[..]);
    // println!("7\tbuffer={:?},fbuffer={:?}", buffer, fbuffer);

    // let mut buffer = [0; 2];
    // let mut fbuffer = [0; 2];
    // let cnt = read.seek(SeekFrom::End(-2)).unwrap();
    // fread.seek(SeekFrom::End(-2));
    // let fcnt = fread.read(&mut fbuffer[..]).unwrap();
    // println!(
    //     "8\tbuffer={:?},cnt={},fbuffer={:?},fcnt={}",
    //     buffer, cnt, fbuffer, fcnt
    // );
    // //Start(79501248)
    // let fname = std::path::Path::new("/tmp/ArmsAgent.zip");
    // let file = fs::File::open(fname).unwrap();
    // let mut fread = BufReader::new(file);

    // println!("\n\n");

    let mut read = HttpReader::new(String::from(
        "http://arms-apm-cn-hangzhou.oss-cn-hangzhou.aliyuncs.com/2.8.0-luyanbo/ArmsAgent.zip",
    ));
    list_zip_contents(read);
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

    fn getBytes(&mut self, cache_index: usize) {
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
        println!("read bytes={}-{} bytes", range_start, range_end,);
    }
}

impl Read for HttpReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        for pos in self.cur_pos..(self.cur_pos + buf.len()) {
            let cache_index = pos / 512 * 512;
            if !self.buffer.contains_key(&cache_index) {
                self.getBytes(cache_index);
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
        println!("seek {:?}, self.cur_pos = {}", pos, self.cur_pos);
        return Ok(self.cur_pos as u64);
    }
}

fn list_zip_contents(reader: impl Read + Seek) -> zip::result::ZipResult<()> {
    let mut zip = zip::ZipArchive::new(reader)?;

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        println!("Filename: {}", file.name());
        break;
    }
    let mut version_f = zip.by_name("ArmsAgent/version").unwrap();
    let mut version_content = String::new();
    version_f.read_to_string(&mut version_content).unwrap();
    print!("version:\n\n{}\n\n", version_content);

    Ok(())
}
