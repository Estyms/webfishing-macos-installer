use binary_reader::{BinaryReader, Endian};

#[derive(Debug)]
pub struct PCK {
    version: u8,
    major_version: u8,
    minor_version: u8,
    patch_version: u8,
    files: Vec<PckFile>,
}

#[derive(Debug, Clone)]
pub struct PckFile {
    path: String,
    offset: u64,
    content: Vec<u8>,
    md5: [u8; 16],
}

impl PCK {
    /// Converts pck file bytes to a PCK struct
    pub fn from_bytes(bytes: &[u8]) -> Result<PCK, &str> {
        let chunked_start = bytes.chunks(16).position(|chunk| chunk.starts_with("GDPC".as_ref()));
        if let None = chunked_start {
            return Err("Invalid PCK");
        };

        let start_offset = chunked_start.unwrap() * 16;
        let pck: Vec<u8> = bytes.to_vec().into_iter().skip(start_offset).collect();
        let mut pck_reader = BinaryReader::from_vec(&pck);
        pck_reader.set_endian(Endian::Little);

        let _magic = pck_reader.read_u32().unwrap();
        let version = pck_reader.read_u32().unwrap();
        let major_version = pck_reader.read_u32().unwrap();
        let minor_version = pck_reader.read_u32().unwrap();
        let patch_version = pck_reader.read_u32().unwrap();

        // Skip unused
        let _ = pck_reader.read(64);

        let num_files = pck_reader.read_u32().unwrap();
        let mut files = vec![];
        for _ in 0..num_files {
            files.push(PckFile::from_bytes(&mut pck_reader, bytes));
        }

        Ok(PCK {
            version: version as u8,
            major_version: major_version as u8,
            minor_version: minor_version as u8,
            patch_version: patch_version as u8,
            files
        })
    }

    /// Converts a PCK struct to the byte vector representing a pck file
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.append(&mut "GDPC".as_bytes().to_vec());
        bytes.append((self.version as u32).to_le_bytes().to_vec().as_mut());
        bytes.append((self.major_version as u32).to_le_bytes().to_vec().as_mut());
        bytes.append((self.minor_version as u32).to_le_bytes().to_vec().as_mut());
        bytes.append((self.patch_version as u32).to_le_bytes().to_vec().as_mut());
        bytes.append([0u8; 16*4].to_vec().as_mut());
        bytes.append((self.files.len() as u32).to_le_bytes().to_vec().as_mut());

        let mut file_offset = bytes.len()
            + self.files.len() * (4 + 8 + 8 + 16)
            + self.files.iter().map(|x| x.path.len() + (4 - (x.path.len() % 4))).sum::<usize>();

        if file_offset % 16 != 0 {
            file_offset = file_offset + (16 - (file_offset % 16));
        }

        let file_base_offset = file_offset;

        let mut content : Vec<u8> = vec![];

        for file in &self.files {
            content.extend(&file.content);

            let mut padded_content_len = file.content.len();
            if padded_content_len % 16 != 0 {
                padded_content_len = padded_content_len + (16 - (padded_content_len % 16));
            }

            for _ in file.content.len()..padded_content_len {
                content.push(0);
            }

            let padded_length = file.path.len() + (4 - (file.path.len() % 4));

            bytes.append((padded_length as u32).to_le_bytes().to_vec().as_mut());
            bytes.append(file.path.as_bytes().to_vec().as_mut());
            for _ in file.path.len()..padded_length {
                bytes.push(0);
            }
            bytes.append((file_offset as u64).to_le_bytes().to_vec().as_mut());
            bytes.append((padded_content_len as u64).to_le_bytes().to_vec().as_mut());
            bytes.extend(file.md5);

            file_offset += padded_content_len;
        }

        for _ in bytes.len()..file_base_offset {
            bytes.push(0);
        }

        bytes.extend(content);
        bytes
    }

    pub fn get_file_by_path(&self, path: &str) -> Option<&PckFile> {
        self.files.iter().find(|x| x.path == path)
    }

    pub fn get_file_by_path_mut(&mut self, path: &str) -> Option<&mut PckFile> {
        self.files.iter_mut().find(|x| x.path == path)
    }

    pub fn add_file(&mut self, new_file: PckFile) {
        self.files.retain(|x| x.path != new_file.path);
        self.files.push(new_file);
    }
}

impl PckFile {
    pub fn from_bytes(pck_reader: &mut BinaryReader, file_bytes: &[u8]) -> PckFile {
        let path_length = pck_reader.read_u32().unwrap();
        let path_bytes= pck_reader.read(path_length as usize).unwrap();
        let path: String = String::from_utf8_lossy(path_bytes).replace("\0", "").to_string();
        let offset = pck_reader.read_u64().unwrap();
        let size = pck_reader.read_u64().unwrap();
        let md5 = pck_reader.read(16).unwrap();
        let content: Vec<u8> = file_bytes.iter().skip(offset as usize).take(size as usize).cloned().collect();
        PckFile {
            path,
            offset,
            content,
            md5: <[u8; 16]>::try_from(md5).unwrap(),
        }
    }

    pub fn get_content(&self) -> &[u8] {
        &self.content
    }

    pub fn set_content(&mut self, content: Vec<u8>) {
        self.md5 = *md5::compute(&content);
        self.content = content;
    }
}